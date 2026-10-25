use std::collections::HashMap;

use hnsw_common::error::SchemaError;
use hnsw_common::schema::{CollectionSchema, Metric, Payload, VectorId, VectorRecord};
use parking_lot::RwLock;

use crate::error::StorageError;
use crate::store::VectorStore;

#[derive(Debug, Clone)]
struct SlotRecord {
    id: VectorId,
    payload: Payload,
    version: u64,
    deleted: bool,
}

struct InnerStore {
    arena: Vec<f32>,
    slots: Vec<SlotRecord>,
    id_to_slot: HashMap<VectorId, usize>,
    live_count: usize,
}

/// In-memory contiguous vector storage indexed by a hash map of VectorId to arena slot.
///
/// Vectors reside in a single contiguous `Vec<f32>` buffer for memory locality and
/// SIMD vectorization. Concurrency is guarded by an internal reader-writer lock.
pub struct InMemoryVectorStore {
    schema: CollectionSchema,
    inner: RwLock<InnerStore>,
}

impl InMemoryVectorStore {
    /// Construct a new in-memory store configured with the given collection schema.
    pub fn new(schema: CollectionSchema) -> Self {
        Self {
            schema,
            inner: RwLock::new(InnerStore {
                arena: Vec::new(),
                slots: Vec::new(),
                id_to_slot: HashMap::new(),
                live_count: 0,
            }),
        }
    }

    /// Retrieve the schema governing this vector store.
    pub fn schema(&self) -> &CollectionSchema {
        &self.schema
    }

    fn validate_record(&self, record: &VectorRecord) -> Result<(), StorageError> {
        let expected_dim = self.schema.dimension.get() as usize;
        if record.vector.len() != expected_dim {
            return Err(StorageError::DimensionMismatch {
                expected: expected_dim,
                actual: record.vector.len(),
            });
        }
        for &val in &record.vector {
            if !val.is_finite() {
                return Err(StorageError::Schema(SchemaError::NonFiniteFloat));
            }
        }
        if self.schema.metric == Metric::Cosine && record.vector.iter().all(|&x| x == 0.0) {
            return Err(StorageError::Schema(SchemaError::ZeroVectorForCosine));
        }
        Ok(())
    }
}

impl VectorStore for InMemoryVectorStore {
    fn put(&self, record: VectorRecord) -> Result<(), StorageError> {
        self.validate_record(&record)?;

        let mut inner = self.inner.write();
        if inner.id_to_slot.contains_key(&record.id) {
            return Err(StorageError::DuplicateId(record.id));
        }

        let slot = inner.slots.len();
        inner.arena.extend_from_slice(&record.vector);

        let is_live = !record.deleted;
        if is_live {
            inner.live_count += 1;
        }

        inner.slots.push(SlotRecord {
            id: record.id,
            payload: record.payload,
            version: record.version,
            deleted: record.deleted,
        });
        inner.id_to_slot.insert(record.id, slot);

        Ok(())
    }

    fn get(&self, id: VectorId) -> Option<VectorRecord> {
        let inner = self.inner.read();
        let slot = *inner.id_to_slot.get(&id)?;
        let metadata = &inner.slots[slot];
        if metadata.deleted {
            return None;
        }

        let dim = self.schema.dimension.get() as usize;
        let start = slot * dim;
        let end = start + dim;
        let vector = inner.arena[start..end].to_vec();

        Some(VectorRecord {
            schema_version: self.schema.schema_version,
            id: metadata.id,
            vector,
            payload: metadata.payload.clone(),
            version: metadata.version,
            deleted: metadata.deleted,
        })
    }

    fn len(&self) -> usize {
        self.inner.read().live_count
    }

    fn iter(&self) -> Vec<VectorRecord> {
        let inner = self.inner.read();
        let dim = self.schema.dimension.get() as usize;
        let mut records = Vec::with_capacity(inner.live_count);

        for (slot, meta) in inner.slots.iter().enumerate() {
            if !meta.deleted {
                let start = slot * dim;
                let end = start + dim;
                records.push(VectorRecord {
                    schema_version: self.schema.schema_version,
                    id: meta.id,
                    vector: inner.arena[start..end].to_vec(),
                    payload: meta.payload.clone(),
                    version: meta.version,
                    deleted: meta.deleted,
                });
            }
        }

        records
    }

    fn memory_bytes(&self) -> usize {
        let inner = self.inner.read();
        let arena_bytes = inner.arena.capacity() * std::mem::size_of::<f32>();
        let slots_bytes = inner.slots.capacity() * std::mem::size_of::<SlotRecord>();
        let map_bytes = inner.id_to_slot.capacity()
            * (std::mem::size_of::<VectorId>() + std::mem::size_of::<usize>());
        arena_bytes + slots_bytes + map_bytes + std::mem::size_of::<InnerStore>()
    }
}
