use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use hnsw_api::server::{build_router, serve, AppState};
use hnsw_common::config::Config;
use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use hnsw_service::error::ServiceError;
use hnsw_service::filter::NoFilter;
use hnsw_service::response::ResponseBuilder;
use hnsw_service::retriever::Retriever;
use hnsw_service::service::SearchService;
use hnsw_service::validator::Validator;

/// Spy retriever counting invocations.
struct SpyRetriever {
    calls: Arc<AtomicUsize>,
    records: Vec<VectorRecord>,
}

impl Retriever for SpyRetriever {
    fn retrieve(&self, _vector: &[f32], _k: usize) -> Result<Vec<VectorRecord>, ServiceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.records.clone())
    }
}

fn test_state(calls: Arc<AtomicUsize>) -> Arc<AppState<SpyRetriever, NoFilter>> {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::L2,
    };
    let config = Config {
        max_body_bytes: 1024, // small for test
        ..Config::default()
    };
    let validator = Validator::new(schema.clone(), config.clone());

    let mut payload = Payload::default();
    payload
        .0
        .insert("title".into(), PayloadValue::String("item-1".into()));

    let rec1 = VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2, 0.3], payload).unwrap();

    let retriever = SpyRetriever {
        calls,
        records: vec![rec1],
    };

    let service = SearchService::new(
        schema,
        config.clone(),
        validator,
        retriever,
        NoFilter,
        ResponseBuilder,
    );
    Arc::new(AppState {
        service: Arc::new(service),
        config,
    })
}

#[tokio::test]
async fn test_search_api_contracts_and_rejections() {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = test_state(Arc::clone(&calls));
    let app = build_router(state);

    // 1. Successful POST /v1/search
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "include_payload": true}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("results").is_some());
    assert!(json.get("took_ms").is_some());
    let results = json["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], 1);
    assert_eq!(results[0]["payload"]["title"], "item-1");

    // 2. Unknown fields -> 400
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "unknown_extra": 123}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "unknown_field");

    // 3. Filter not an object (e.g. string) -> 400
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "filter": "invalid_string"}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "invalid_filter");

    // 4. Valid filter object -> 200 OK and reaches SearchService
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "filter": {"must": [{"key": "title", "match": {"value": "item-1"}}]}}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4b. Malformed filter -> 400 with key named in the message
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "filter": {"must": [{"key": "price", "range": {"gt": "invalid"}}]}}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "invalid_range_bound");
    assert!(json["error"]["message"].as_str().unwrap().contains("price"));

    // 4c. Over-limit filter (depth > max) -> 400
    let mut deep_filter = serde_json::json!({"key": "a", "match": {"value": 1}});
    for _ in 0..15 {
        deep_filter = serde_json::json!({"must": [deep_filter]});
    }
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "vector": [0.1, 0.2, 0.3],
                "k": 5,
                "filter": deep_filter
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "max_filter_depth_exceeded");

    // 5. Malformed JSON -> 400
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"vector": [0.1, 0.2,"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "malformed_json");

    // 6. Wrong Content-Type -> 415
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(r#"{"vector": [0.1, 0.2, 0.3], "k": 5}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "unsupported_media_type");

    // 7. Oversized body -> 413 (our test limit is 1024 bytes)
    let big_vec: Vec<f32> = vec![0.1; 500];
    let big_json = serde_json::json!({
        "vector": big_vec,
        "k": 5
    })
    .to_string();
    assert!(big_json.len() > 1024);
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(big_json))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "payload_too_large");

    // 8. Unknown route -> 404 JSON
    let req = Request::builder()
        .method("POST")
        .uri("/v1/unknown")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "not_found");

    // 9. Wrong method -> 405 JSON
    let req = Request::builder()
        .method("GET")
        .uri("/v1/search")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "method_not_allowed");
}

#[tokio::test]
async fn test_golden_contract_files() {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = test_state(calls);
    let app = build_router(state);

    let golden_req = include_str!("golden/search_request_valid.json");
    let golden_resp = include_str!("golden/search_response_valid.json");

    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(golden_req))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let actual_json: Value = serde_json::from_slice(&bytes).unwrap();
    let expected_json: Value = serde_json::from_str(golden_resp).unwrap();

    assert_eq!(actual_json["results"], expected_json["results"]);
    assert!(actual_json["took_ms"].is_f64());
}

#[tokio::test]
async fn test_handler_has_no_business_logic() {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = test_state(Arc::clone(&calls));
    let app = build_router(state);

    // Invalid query vector (dimension 2 vs expected 3)
    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"vector": [0.1, 0.2], "k": 5}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "dimension_mismatch");

    // Retriever was never called!
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    // Valid query
    let valid_req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"vector": [0.1, 0.2, 0.3], "k": 5}"#))
        .unwrap();
    let valid_resp = app.oneshot(valid_req).await.unwrap();
    assert_eq!(valid_resp.status(), StatusCode::OK);

    // Retriever was called exactly once!
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_serve_graceful_shutdown() {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = test_state(calls);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        serve(listener, app, async move {
            let _ = shutdown_rx.await;
        })
        .await
    });

    // Make request to the live server
    let client = reqwest_or_hyper_get(addr).await;
    assert!(client);

    // Signal shutdown
    let _ = shutdown_tx.send(());

    // Assert serve returns within 2 seconds
    let shutdown_result = tokio::time::timeout(Duration::from_secs(2), server_task).await;
    assert!(shutdown_result.is_ok(), "server must shut down within 2s");
}

async fn reqwest_or_hyper_get(addr: std::net::SocketAddr) -> bool {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "POST /v1/search HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: 35\r\nConnection: close\r\n\r\n{{\"vector\": [0.1, 0.2, 0.3], \"k\": 5}}",
        addr
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let resp = String::from_utf8_lossy(&buf[..n]);
    resp.starts_with("HTTP/1.1 200 OK")
}

#[tokio::test]
async fn test_missing_content_type_rejected() {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = test_state(calls);
    let app = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .body(Body::from(r#"{"vector": [0.1, 0.2, 0.3], "k": 5}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "unsupported_media_type");
}

#[test]
fn test_empty_index_retriever() {
    use hnsw_api::empty_retriever::EmptyIndexRetriever;
    let retriever = EmptyIndexRetriever;
    let res = retriever.retrieve(&[1.0, 2.0], 10).unwrap();
    assert!(res.is_empty());
}

#[tokio::test]
async fn test_all_api_error_mappings() {
    use axum::response::IntoResponse;
    use hnsw_api::error::ApiError;
    use hnsw_common::error::SchemaError;
    use hnsw_service::error::ServiceError;

    let cases: Vec<(ApiError, StatusCode, &str)> = vec![
        (
            ApiError::MalformedJson("bad".into()),
            StatusCode::BAD_REQUEST,
            "malformed_json",
        ),
        (
            ApiError::UnknownField("bad".into()),
            StatusCode::BAD_REQUEST,
            "unknown_field",
        ),
        (
            ApiError::InvalidFilter,
            StatusCode::BAD_REQUEST,
            "invalid_filter",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::UnknownField("f".into())),
            StatusCode::BAD_REQUEST,
            "unknown_field",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::TypeMismatch {
                key: "k".into(),
                expected: "num",
                actual: "str",
            }),
            StatusCode::BAD_REQUEST,
            "filter_type_error",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::MixedTypeList { key: "k".into() }),
            StatusCode::BAD_REQUEST,
            "mixed_type_list",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::InvalidRangeBound {
                key: "k".into(),
                reason: "nan",
            }),
            StatusCode::BAD_REQUEST,
            "invalid_range_bound",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::ConflictingBounds {
                key: "k".into(),
                reason: "both",
            }),
            StatusCode::BAD_REQUEST,
            "conflicting_bounds",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::MaxDepthExceeded {
                depth: 10,
                max: 5,
            }),
            StatusCode::BAD_REQUEST,
            "max_filter_depth_exceeded",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::MaxClausesExceeded {
                clauses: 10,
                max: 5,
            }),
            StatusCode::BAD_REQUEST,
            "max_filter_clauses_exceeded",
        ),
        (
            ApiError::Filter(hnsw_filter::error::FilterError::InvalidFormat("fmt".into())),
            StatusCode::BAD_REQUEST,
            "invalid_filter",
        ),
        (
            ApiError::UnsupportedMediaType,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
        ),
        (
            ApiError::PayloadTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
        ),
        (ApiError::NotFound, StatusCode::NOT_FOUND, "not_found"),
        (
            ApiError::MethodNotAllowed,
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
        ),
        (
            ApiError::Service(ServiceError::InvalidK { k: 0, max_k: 100 }),
            StatusCode::BAD_REQUEST,
            "invalid_k",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::DimensionZero)),
            StatusCode::BAD_REQUEST,
            "invalid_dimension",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::DimensionTooLarge {
                dim: 5000,
                max: 4096,
            })),
            StatusCode::BAD_REQUEST,
            "dimension_too_large",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::DimensionMismatch {
                expected: 128,
                actual: 64,
            })),
            StatusCode::BAD_REQUEST,
            "dimension_mismatch",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::NonFiniteFloat)),
            StatusCode::BAD_REQUEST,
            "non_finite_float",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::ZeroVectorForCosine)),
            StatusCode::BAD_REQUEST,
            "zero_vector_for_cosine",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::UnsupportedVersion(99))),
            StatusCode::BAD_REQUEST,
            "unsupported_schema_version",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::AlreadyDeleted(VectorId(
                1,
            )))),
            StatusCode::BAD_REQUEST,
            "already_deleted",
        ),
        (
            ApiError::Service(ServiceError::Schema(SchemaError::VersionOverflow)),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        ),
        (
            ApiError::Service(ServiceError::Retrieval("db error".into())),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        ),
        (
            ApiError::Service(ServiceError::Filter("filter failed".into())),
            StatusCode::BAD_REQUEST,
            "filter_error",
        ),
    ];

    for (err, expected_status, expected_code) in cases {
        let resp = err.into_response();
        assert_eq!(resp.status(), expected_status);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"]["code"], expected_code);
    }
}

#[tokio::test]
async fn test_spy_retriever_filter_excludes_rejected_records() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::L2,
    };
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());

    let mut p1 = Payload::default();
    p1.0.insert("category".into(), PayloadValue::String("books".into()));
    let rec1 = VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2, 0.3], p1).unwrap();

    let mut p2 = Payload::default();
    p2.0.insert(
        "category".into(),
        PayloadValue::String("electronics".into()),
    );
    let rec2 = VectorRecord::new(&schema, VectorId(2), vec![0.1, 0.2, 0.3], p2).unwrap();

    let calls = Arc::new(AtomicUsize::new(0));
    let retriever = SpyRetriever {
        calls: Arc::clone(&calls),
        records: vec![rec1, rec2],
    };

    let service = SearchService::new(
        schema,
        config.clone(),
        validator,
        retriever,
        NoFilter,
        ResponseBuilder,
    );
    let state = Arc::new(AppState {
        service: Arc::new(service),
        config,
    });
    let app = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"vector": [0.1, 0.2, 0.3], "k": 5, "filter": {"must": [{"key": "category", "match": {"value": "books"}}]}, "include_payload": true}"#,
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();

    let results = json["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], 1);
    assert_eq!(results[0]["payload"]["category"], "books");

    // Proves retriever was called exactly once and rejected records were excluded
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_search_api_128_dim_vector_returns_empty_results() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(128, 4096).unwrap(),
        metric: Metric::Cosine,
    };
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());
    let retriever = hnsw_api::empty_retriever::EmptyIndexRetriever;
    let service = SearchService::new(
        schema,
        config.clone(),
        validator,
        retriever,
        NoFilter,
        ResponseBuilder,
    );
    let state = Arc::new(AppState {
        service: Arc::new(service),
        config,
    });
    let app = build_router(state);

    let query_vector: Vec<f32> = vec![0.1; 128];
    let body = serde_json::json!({
        "vector": query_vector,
        "k": 5
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/search")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();

    let results = json["results"].as_array().unwrap();
    assert_eq!(results.len(), 0);
    assert!(json["took_ms"].is_f64());
}

