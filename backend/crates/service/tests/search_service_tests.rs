use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use hnsw_common::config::Config;
use hnsw_common::error::SchemaError;
use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use hnsw_service::error::ServiceError;
use hnsw_service::filter::{FilterEvaluator, NoFilter};
use hnsw_service::response::ResponseBuilder;
use hnsw_service::retriever::Retriever;
use hnsw_service::service::{SearchRequest, SearchService};
use hnsw_service::validator::Validator;

/// Test-only fake retriever demonstrating the search pipeline.
struct FakeRetriever {
    records: Vec<VectorRecord>,
}

impl Retriever for FakeRetriever {
    fn retrieve(&self, _vector: &[f32], _k: usize) -> Result<Vec<VectorRecord>, ServiceError> {
        Ok(self.records.clone())
    }
}

/// Spy retriever that tracks if it was called and can optionally inject errors.
struct SpyRetriever {
    called: Arc<AtomicBool>,
    fail_with: Option<String>,
    records: Vec<VectorRecord>,
}

impl Retriever for SpyRetriever {
    fn retrieve(&self, _vector: &[f32], _k: usize) -> Result<Vec<VectorRecord>, ServiceError> {
        self.called.store(true, Ordering::SeqCst);
        if let Some(err_msg) = &self.fail_with {
            return Err(ServiceError::Retrieval(err_msg.clone()));
        }
        Ok(self.records.clone())
    }
}

struct OddIdOnlyFilter;

impl FilterEvaluator for OddIdOnlyFilter {
    fn matches(&self, record: &VectorRecord) -> Result<bool, ServiceError> {
        Ok(record.id.0 % 2 == 1)
    }
}

fn sample_schema() -> CollectionSchema {
    CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::L2,
    }
}

#[test]
fn test_search_service_composition_and_api() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());
    let filter = NoFilter;
    let response_builder = ResponseBuilder;

    let rec1 = VectorRecord::new(
        &schema,
        VectorId(1),
        vec![1.0, 0.0, 0.0],
        Payload::default(),
    )
    .unwrap();
    let retriever = FakeRetriever {
        records: vec![rec1],
    };

    let service = SearchService::new(
        schema.clone(),
        config.clone(),
        validator,
        retriever,
        filter,
        response_builder,
    );
    assert_eq!(service.schema(), &schema);
    assert_eq!(service.config(), &config);

    let req = SearchRequest {
        vector: vec![1.0, 0.0, 0.0],
        k: 10,
        filter: None,
        include_payload: true,
    };
    let res = service.search(req).expect("search should succeed");
    assert_eq!(res.results.len(), 1);
    assert_eq!(res.results[0].id, VectorId(1));
}

#[test]
fn test_validator_unit_independent() {
    let schema = sample_schema();
    let config = Config {
        max_k: 50,
        ..Config::default()
    };
    let validator = Validator::new(schema, config);

    // Rejects k == 0
    let err_k0 = validator
        .validate(&SearchRequest {
            vector: vec![1.0, 2.0, 3.0],
            k: 0,
            filter: None,
            include_payload: false,
        })
        .unwrap_err();
    assert!(matches!(err_k0, ServiceError::InvalidK { k: 0, max_k: 50 }));

    // Rejects k > max_k
    let err_k_large = validator
        .validate(&SearchRequest {
            vector: vec![1.0, 2.0, 3.0],
            k: 51,
            filter: None,
            include_payload: false,
        })
        .unwrap_err();
    assert!(matches!(
        err_k_large,
        ServiceError::InvalidK { k: 51, max_k: 50 }
    ));

    // Rejects dimension mismatch
    let err_dim = validator
        .validate(&SearchRequest {
            vector: vec![1.0, 2.0], // dim 2 vs expected 3
            k: 5,
            filter: None,
            include_payload: false,
        })
        .unwrap_err();
    assert!(matches!(
        err_dim,
        ServiceError::Schema(SchemaError::DimensionMismatch {
            expected: 3,
            actual: 2
        })
    ));

    // Rejects non-finite floats
    let err_nan = validator
        .validate(&SearchRequest {
            vector: vec![1.0, f32::NAN, 3.0],
            k: 5,
            filter: None,
            include_payload: false,
        })
        .unwrap_err();
    assert!(matches!(
        err_nan,
        ServiceError::Schema(SchemaError::NonFiniteFloat)
    ));

    // Rejects all-zero vector for Cosine metric
    let cosine_schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::Cosine,
    };
    let cosine_validator = Validator::new(cosine_schema, Config::default());
    let err_zero_cosine = cosine_validator
        .validate(&SearchRequest {
            vector: vec![0.0, 0.0, 0.0],
            k: 5,
            filter: None,
            include_payload: false,
        })
        .unwrap_err();
    assert!(matches!(
        err_zero_cosine,
        ServiceError::Schema(SchemaError::ZeroVectorForCosine)
    ));
}

#[test]
fn test_filter_evaluator_unit_independent() {
    let schema = sample_schema();
    let rec = VectorRecord::new(
        &schema,
        VectorId(1),
        vec![1.0, 2.0, 3.0],
        Payload::default(),
    )
    .unwrap();
    let no_filter = NoFilter;
    assert!(no_filter.matches(&rec).unwrap());
}

#[test]
fn test_response_builder_unit_independent() {
    let builder = ResponseBuilder;
    let schema = sample_schema();

    let mut payload = Payload::default();
    payload
        .0
        .insert("name".into(), PayloadValue::String("test".into()));

    let rec1 =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 0.0, 0.0], payload.clone()).unwrap();
    let rec2 = VectorRecord::new(
        &schema,
        VectorId(2),
        vec![0.0, 1.0, 0.0],
        Payload::default(),
    )
    .unwrap();

    let query = vec![1.0, 0.0, 0.0];
    let records = vec![rec1.clone(), rec2.clone()];

    // With L2
    let res_l2 = builder.build(records.clone(), 10, true, Metric::L2, &query, 1.25);
    assert_eq!(res_l2.results.len(), 2);
    assert_eq!(res_l2.results[0].id, VectorId(1));
    assert_eq!(res_l2.results[0].payload, Some(payload));
    assert!(res_l2.results[0].score >= res_l2.results[1].score);
    assert_eq!(res_l2.took_ms, 1.25);

    // With Cosine
    let res_cosine = builder.build(records.clone(), 10, false, Metric::Cosine, &query, 1.0);
    assert_eq!(res_cosine.results.len(), 2);
    assert!((res_cosine.results[0].score - 1.0).abs() < 1e-5); // rec1 has identical angle to query
    assert!((res_cosine.results[1].score - 0.0).abs() < 1e-5); // rec2 is orthogonal

    // Cosine zero vector in records handles gracefully
    let zero_score = ResponseBuilder::build(
        &builder,
        vec![rec1.clone()],
        1,
        false,
        Metric::Cosine,
        &[0.0, 0.0, 0.0],
        0.1,
    );
    assert_eq!(zero_score.results[0].score, 0.0);

    // With InnerProduct
    let res_ip = builder.build(
        records.clone(),
        10,
        false,
        Metric::InnerProduct,
        &query,
        0.8,
    );
    assert_eq!(res_ip.results[0].score, 1.0);

    // With include_payload = false and k = 1 truncation
    let res_truncated = builder.build(records, 1, false, Metric::L2, &query, 0.5);
    assert_eq!(res_truncated.results.len(), 1);
    assert_eq!(res_truncated.results[0].payload, None);
}

#[test]
fn test_fake_retriever_drives_end_to_end_flow() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());
    let filter = NoFilter;
    let response_builder = ResponseBuilder;

    let rec1 = VectorRecord::new(
        &schema,
        VectorId(10),
        vec![0.1, 0.2, 0.3],
        Payload::default(),
    )
    .unwrap();
    let rec2 = VectorRecord::new(
        &schema,
        VectorId(20),
        vec![0.4, 0.5, 0.6],
        Payload::default(),
    )
    .unwrap();
    let fake_retriever = FakeRetriever {
        records: vec![rec1, rec2],
    };

    let service = SearchService::new(
        schema,
        config,
        validator,
        fake_retriever,
        filter,
        response_builder,
    );
    let req = SearchRequest {
        vector: vec![0.1, 0.2, 0.3],
        k: 1,
        filter: None,
        include_payload: false,
    };
    let response = service.search(req).unwrap();
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].id, VectorId(10));
}

#[test]
fn test_no_filter_evaluator_matches_all() {
    let schema = sample_schema();
    let mut rec = VectorRecord::new(
        &schema,
        VectorId(1),
        vec![1.0, 1.0, 1.0],
        Payload::default(),
    )
    .unwrap();
    let filter = NoFilter;
    assert!(filter.matches(&rec).unwrap());

    rec.tombstone().unwrap();
    assert!(filter.matches(&rec).unwrap());
}

#[test]
fn test_service_error_mapping() {
    let schema_err = SchemaError::DimensionZero;
    let service_err: ServiceError = schema_err.into();
    assert!(matches!(
        service_err,
        ServiceError::Schema(SchemaError::DimensionZero)
    ));

    let k_err = ServiceError::InvalidK { k: 0, max_k: 100 };
    assert_eq!(
        k_err.to_string(),
        "invalid k: k must be greater than 0 and at most 100, got 0"
    );
}

// Required Part B tests:
// (i) validator rejection becomes a ServiceError and the Retriever is never called
#[test]
fn test_validator_rejection_prevents_retriever_call() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());
    let called = Arc::new(AtomicBool::new(false));
    let spy = SpyRetriever {
        called: Arc::clone(&called),
        fail_with: None,
        records: vec![],
    };
    let service = SearchService::new(schema, config, validator, spy, NoFilter, ResponseBuilder);

    // Invalid request (k = 0)
    let invalid_req = SearchRequest {
        vector: vec![1.0, 2.0, 3.0],
        k: 0,
        filter: None,
        include_payload: false,
    };
    let err = service.search(invalid_req).unwrap_err();
    assert!(matches!(err, ServiceError::InvalidK { .. }));
    // Retriever was never called!
    assert!(!called.load(Ordering::SeqCst));
}

// (ii) a Retriever error propagates as ServiceError
#[test]
fn test_retriever_error_propagates_as_service_error() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());
    let called = Arc::new(AtomicBool::new(false));
    let spy = SpyRetriever {
        called: Arc::clone(&called),
        fail_with: Some("disk IO failure on index".into()),
        records: vec![],
    };
    let service = SearchService::new(schema, config, validator, spy, NoFilter, ResponseBuilder);

    let req = SearchRequest {
        vector: vec![1.0, 2.0, 3.0],
        k: 5,
        filter: None,
        include_payload: false,
    };
    let err = service.search(req).unwrap_err();
    assert_eq!(
        err,
        ServiceError::Retrieval("disk IO failure on index".into())
    );
    assert!(called.load(Ordering::SeqCst));
}

// (iii) records rejected by FilterEvaluator are excluded
#[test]
fn test_records_rejected_by_filter_evaluator_are_excluded() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());

    let rec1 = VectorRecord::new(
        &schema,
        VectorId(1),
        vec![1.0, 0.0, 0.0],
        Payload::default(),
    )
    .unwrap();
    let rec2 = VectorRecord::new(
        &schema,
        VectorId(2),
        vec![0.0, 1.0, 0.0],
        Payload::default(),
    )
    .unwrap();
    let rec3 = VectorRecord::new(
        &schema,
        VectorId(3),
        vec![0.0, 0.0, 1.0],
        Payload::default(),
    )
    .unwrap();
    let retriever = FakeRetriever {
        records: vec![rec1, rec2, rec3],
    };

    // OddIdOnlyFilter rejects record 2 (even ID)
    let service = SearchService::new(
        schema,
        config,
        validator,
        retriever,
        OddIdOnlyFilter,
        ResponseBuilder,
    );

    let req = SearchRequest {
        vector: vec![1.0, 0.0, 0.0],
        k: 10,
        filter: None,
        include_payload: false,
    };
    let res = service.search(req).unwrap();
    let ids: Vec<u64> = res.results.iter().map(|r| r.id.0).collect();
    assert_eq!(ids, vec![1, 3]);
}

// (iv) results are sorted by score and truncated to k
#[test]
fn test_results_sorted_by_score_and_truncated_to_k() {
    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());

    // rec_close is identical to query (score = 1.0)
    let rec_close = VectorRecord::new(
        &schema,
        VectorId(10),
        vec![1.0, 0.0, 0.0],
        Payload::default(),
    )
    .unwrap();
    // rec_mid is distance sq = 1.0 (score = 0.5)
    let rec_mid = VectorRecord::new(
        &schema,
        VectorId(20),
        vec![0.0, 0.0, 0.0],
        Payload::default(),
    )
    .unwrap();
    // rec_far is distance sq = 4.0 (score = 0.2)
    let rec_far = VectorRecord::new(
        &schema,
        VectorId(30),
        vec![-1.0, 0.0, 0.0],
        Payload::default(),
    )
    .unwrap();

    let retriever = FakeRetriever {
        // Returned in scrambled order: far, close, mid
        records: vec![rec_far, rec_close, rec_mid],
    };

    let service = SearchService::new(
        schema,
        config,
        validator,
        retriever,
        NoFilter,
        ResponseBuilder,
    );

    let req = SearchRequest {
        vector: vec![1.0, 0.0, 0.0],
        k: 2, // Only top 2
        filter: None,
        include_payload: false,
    };
    let res = service.search(req).unwrap();

    assert_eq!(res.results.len(), 2);
    // Highest score first (rec_close), then rec_mid
    assert_eq!(res.results[0].id, VectorId(10));
    assert_eq!(res.results[1].id, VectorId(20));
    assert!(res.results[0].score >= res.results[1].score);
}

#[test]
fn test_ast_filter_evaluator_and_filter_matches() {
    use hnsw_filter::parser::FilterParser;
    use hnsw_service::filter::{AstFilterEvaluator, FilterEvaluator};
    use serde_json::json;

    let schema = sample_schema();
    let mut payload = Payload::default();
    payload
        .0
        .insert("category".into(), PayloadValue::String("books".into()));
    let rec = VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2, 0.3], payload).unwrap();

    let config = Config::default();
    let filter = FilterParser::parse(
        &json!({"must": [{"key": "category", "match": {"value": "books"}}]}),
        &config,
    )
    .unwrap();

    // Test AstFilterEvaluator
    let evaluator = AstFilterEvaluator::new(filter.clone());
    assert!(evaluator.matches(&rec).unwrap());

    // Test Filter directly as FilterEvaluator
    assert!(filter.matches(&rec).unwrap());
}

#[test]
fn test_search_service_per_request_filter() {
    use hnsw_filter::parser::FilterParser;
    use serde_json::json;

    let schema = sample_schema();
    let config = Config::default();
    let validator = Validator::new(schema.clone(), config.clone());

    let mut p1 = Payload::default();
    p1.0.insert("tag".into(), PayloadValue::String("target".into()));
    let rec1 = VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2, 0.3], p1).unwrap();

    let mut p2 = Payload::default();
    p2.0.insert("tag".into(), PayloadValue::String("other".into()));
    let rec2 = VectorRecord::new(&schema, VectorId(2), vec![0.1, 0.2, 0.3], p2).unwrap();

    let retriever = FakeRetriever {
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

    let filter = FilterParser::parse(
        &json!({"must": [{"key": "tag", "match": {"value": "target"}}]}),
        &config,
    )
    .unwrap();

    let req = SearchRequest {
        vector: vec![0.1, 0.2, 0.3],
        k: 10,
        filter: Some(filter),
        include_payload: true,
    };

    let resp = service.search(req).unwrap();
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].id, VectorId(1));
}
