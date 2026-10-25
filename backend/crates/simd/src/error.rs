use thiserror::Error;

/// Typed errors arising from vector distance computation and validation.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum MetricError {
    #[error("dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("cosine distance is undefined for a zero vector (norm is zero)")]
    ZeroVector,

    #[error("vector contains non-finite float value (NaN or Infinity)")]
    NonFiniteFloat,
}
