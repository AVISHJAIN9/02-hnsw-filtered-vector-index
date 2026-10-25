//! Vector storage, append-only WAL, memory-mapped checkpoints, and contiguous arenas.

pub mod error;
pub mod in_memory;
pub mod store;

pub use error::StorageError;
pub use in_memory::InMemoryVectorStore;
pub use store::VectorStore;
