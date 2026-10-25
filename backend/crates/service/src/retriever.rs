//! Vector index retrieval abstraction boundary.

use hnsw_common::schema::VectorRecord;

use crate::error::ServiceError;

/// Trait defining the candidate retrieval boundary.
pub trait Retriever: Send + Sync {
    /// Retrieve up to `k` candidate vector records nearest to `vector`.
    fn retrieve(&self, vector: &[f32], k: usize) -> Result<Vec<VectorRecord>, ServiceError>;
}
