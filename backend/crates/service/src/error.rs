//! Typed error definitions for search service operations.

use hnsw_common::error::SchemaError;
use thiserror::Error;

/// Errors arising during query validation, filtering, retrieval, or response assembly.
#[derive(Debug, Error, PartialEq)]
pub enum ServiceError {
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("invalid k: k must be greater than 0 and at most {max_k}, got {k}")]
    InvalidK { k: usize, max_k: usize },

    #[error("retrieval error: {0}")]
    Retrieval(String),

    #[error("filter error: {0}")]
    Filter(String),

    #[error("top-k error: {0}")]
    TopK(#[from] crate::top_k::TopKError),
}
