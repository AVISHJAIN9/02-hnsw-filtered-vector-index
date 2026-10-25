use hnsw_common::schema::{VectorId, VectorRecord};

use crate::error::StorageError;

/// Core interface for vector storage backends.
pub trait VectorStore: Send + Sync {
    /// Insert a record into the store, validating against the collection schema.
    ///
    /// Rejects duplicate IDs with `StorageError::DuplicateId`.
    fn put(&self, record: VectorRecord) -> Result<(), StorageError>;

    /// Retrieve a live record by its `VectorId`.
    fn get(&self, id: VectorId) -> Option<VectorRecord>;

    /// Current count of live (non-deleted) records.
    fn len(&self) -> usize;

    /// Check if the store contains zero live records.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Retrieve an iterator / list of all live records.
    fn iter(&self) -> Vec<VectorRecord>;

    /// Estimate total memory footprint in bytes.
    fn memory_bytes(&self) -> usize;
}
