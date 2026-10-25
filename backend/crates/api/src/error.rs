//! HTTP error definitions and uniform JSON error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hnsw_common::error::SchemaError;
use hnsw_filter::error::FilterError;
use hnsw_service::error::ServiceError;
use serde_json::json;

/// Unified API error type mapping to specific HTTP status codes and stable error codes.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("malformed JSON body: {0}")]
    MalformedJson(String),

    #[error("unknown field: {0}")]
    UnknownField(String),

    #[error("filter must be a JSON object")]
    InvalidFilter,

    #[error("filter error: {0}")]
    Filter(#[from] FilterError),

    #[error("unsupported Content-Type: expected application/json")]
    UnsupportedMediaType,

    #[error("request payload exceeds maximum allowed size")]
    PayloadTooLarge,

    #[error("route not found")]
    NotFound,

    #[error("HTTP method not allowed")]
    MethodNotAllowed,

    #[error("service error: {0}")]
    Service(#[from] ServiceError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::MalformedJson(msg) => {
                (StatusCode::BAD_REQUEST, "malformed_json", msg.clone())
            }
            ApiError::UnknownField(msg) => (StatusCode::BAD_REQUEST, "unknown_field", msg.clone()),
            ApiError::InvalidFilter => (
                StatusCode::BAD_REQUEST,
                "invalid_filter",
                "filter must be a JSON object".to_string(),
            ),
            ApiError::Filter(f) => {
                let code = match f {
                    FilterError::UnknownField(_) => "unknown_field",
                    FilterError::TypeMismatch { .. } => "filter_type_error",
                    FilterError::MixedTypeList { .. } => "mixed_type_list",
                    FilterError::InvalidRangeBound { .. } => "invalid_range_bound",
                    FilterError::ConflictingBounds { .. } => "conflicting_bounds",
                    FilterError::MaxDepthExceeded { .. } => "max_filter_depth_exceeded",
                    FilterError::MaxClausesExceeded { .. } => "max_filter_clauses_exceeded",
                    FilterError::InvalidFormat(_) => "invalid_filter",
                };
                (StatusCode::BAD_REQUEST, code, f.to_string())
            }
            ApiError::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "Content-Type must be application/json".to_string(),
            ),
            ApiError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "request body exceeded maximum allowed bytes".to_string(),
            ),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "endpoint not found".to_string(),
            ),
            ApiError::MethodNotAllowed => (
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "method not allowed for this route".to_string(),
            ),
            ApiError::Service(err) => match err {
                ServiceError::InvalidK { k, max_k } => (
                    StatusCode::BAD_REQUEST,
                    "invalid_k",
                    format!("k must be between 1 and {}, got {}", max_k, k),
                ),
                ServiceError::Schema(schema_err) => match schema_err {
                    SchemaError::DimensionZero => (
                        StatusCode::BAD_REQUEST,
                        "invalid_dimension",
                        "dimension must be greater than zero".to_string(),
                    ),
                    SchemaError::DimensionTooLarge { dim, max } => (
                        StatusCode::BAD_REQUEST,
                        "dimension_too_large",
                        format!("dimension {} exceeds max allowed {}", dim, max),
                    ),
                    SchemaError::DimensionMismatch { expected, actual } => (
                        StatusCode::BAD_REQUEST,
                        "dimension_mismatch",
                        format!(
                            "query dimension mismatch: expected {}, got {}",
                            expected, actual
                        ),
                    ),
                    SchemaError::NonFiniteFloat => (
                        StatusCode::BAD_REQUEST,
                        "non_finite_float",
                        "vector contains non-finite float value (NaN or Inf)".to_string(),
                    ),
                    SchemaError::ZeroVectorForCosine => (
                        StatusCode::BAD_REQUEST,
                        "zero_vector_for_cosine",
                        "cosine metric cannot be computed for an all-zero vector".to_string(),
                    ),
                    SchemaError::UnsupportedVersion(v) => (
                        StatusCode::BAD_REQUEST,
                        "unsupported_schema_version",
                        format!("unsupported schema version: {}", v),
                    ),
                    SchemaError::AlreadyDeleted(_) => (
                        StatusCode::BAD_REQUEST,
                        "already_deleted",
                        "record is already tombstoned".to_string(),
                    ),
                    SchemaError::VersionOverflow => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        "internal version allocation overflow".to_string(),
                    ),
                },
                ServiceError::Retrieval(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "an internal error occurred during candidate retrieval".to_string(),
                ),
                ServiceError::Filter(f) => (
                    StatusCode::BAD_REQUEST,
                    "filter_error",
                    format!("filter error: {}", f),
                ),
            },
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}
