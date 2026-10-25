//! HTTP request handlers for the Search API.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::Json;
use hnsw_filter::parser::FilterParser;
use hnsw_service::filter::FilterEvaluator;
use hnsw_service::retriever::Retriever;

use crate::dto::{ScoredPointDto, SearchRequestDto, SearchResponseDto};
use crate::error::ApiError;
use crate::server::AppState;

/// Handler for POST /v1/search
pub async fn search_handler<R, F>(
    State(state): State<Arc<AppState<R, F>>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<SearchResponseDto>, ApiError>
where
    R: Retriever + Send + Sync + 'static,
    F: FilterEvaluator + Send + Sync + 'static,
{
    // 1. Content-Type validation
    match headers.get(header::CONTENT_TYPE) {
        Some(val) => {
            let s = val.to_str().unwrap_or("");
            if !s.starts_with("application/json") {
                return Err(ApiError::UnsupportedMediaType);
            }
        }
        None => return Err(ApiError::UnsupportedMediaType),
    }

    // 2. Read body with limit
    let bytes = match to_bytes(body, state.config.max_body_bytes).await {
        Ok(b) => b,
        Err(_) => return Err(ApiError::PayloadTooLarge),
    };

    // 3. Deserialize JSON with strict unknown fields check
    let dto: SearchRequestDto = match serde_json::from_slice(&bytes) {
        Ok(d) => d,
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("unknown field") {
                return Err(ApiError::UnknownField(msg));
            } else {
                return Err(ApiError::MalformedJson(msg));
            }
        }
    };

    // 4. Validate and parse filter field
    let parsed_filter = if let Some(ref filter_val) = dto.filter {
        if !filter_val.is_object() {
            return Err(ApiError::InvalidFilter);
        }
        let ast = FilterParser::parse(filter_val, &state.config)?;
        Some(ast)
    } else {
        None
    };

    // 5. Delegate to SearchService (no business logic in handler)
    let service_req = hnsw_service::service::SearchRequest {
        vector: dto.vector,
        k: dto.k,
        filter: parsed_filter,
        include_payload: dto.include_payload,
    };
    let service_resp = state.service.search(service_req)?;

    let results = service_resp
        .results
        .into_iter()
        .map(|pt| ScoredPointDto {
            id: pt.id.0,
            score: pt.score,
            payload: pt.payload.map(|p| p.0),
        })
        .collect();

    Ok(Json(SearchResponseDto {
        results,
        took_ms: service_resp.took_ms,
    }))
}

/// Fallback handler for unmatched routes (404 Not Found)
pub async fn not_found_handler() -> ApiError {
    ApiError::NotFound
}

/// Fallback handler for unallowed HTTP methods (405 Method Not Allowed)
pub async fn method_not_allowed_handler() -> ApiError {
    ApiError::MethodNotAllowed
}
