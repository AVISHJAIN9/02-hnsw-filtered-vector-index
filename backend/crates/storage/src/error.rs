use hnsw_common::error::SchemaError;
use hnsw_common::schema::VectorId;
use thiserror::Error;

/// Errors arising from vector store operations.
#[derive(Debug, Error, PartialEq)]
pub enum StorageError {
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),

    #[error("record with id {0:?} already exists in store")]
    DuplicateId(VectorId),

    #[error("record dimension {actual} does not match store dimension {expected}")]
    DimensionMismatch { expected: usize, actual: usize },
}
