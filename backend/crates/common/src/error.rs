use crate::schema::VectorId;
use thiserror::Error;

/// Errors arising from vector schema operations and lifecycle transitions.
#[derive(Debug, Error, PartialEq)]
pub enum SchemaError {
    #[error("dimension must be greater than zero")]
    DimensionZero,

    #[error("dimension {dim} exceeds maximum allowed dimension {max}")]
    DimensionTooLarge { dim: u32, max: u32 },

    #[error("vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("vector contains non-finite float value (NaN or Infinity)")]
    NonFiniteFloat,

    #[error("cosine metric cannot be computed for an all-zero vector")]
    ZeroVectorForCosine,

    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),

    #[error("record with id {0:?} is already deleted")]
    AlreadyDeleted(VectorId),

    #[error("record version counter overflowed")]
    VersionOverflow,
}
