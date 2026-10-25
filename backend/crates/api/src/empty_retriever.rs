//! Retriever returning zero candidates for startup before G2 index is wired.

use hnsw_common::schema::VectorRecord;
use hnsw_service::error::ServiceError;
use hnsw_service::retriever::Retriever;

/// An index retriever over zero vectors that immediately returns empty candidate list.
#[derive(Debug, Clone, Default)]
pub struct EmptyIndexRetriever;

impl Retriever for EmptyIndexRetriever {
    fn retrieve(&self, _vector: &[f32], _k: usize) -> Result<Vec<VectorRecord>, ServiceError> {
        Ok(Vec::new())
    }
}
