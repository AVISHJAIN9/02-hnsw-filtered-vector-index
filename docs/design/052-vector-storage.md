# Design Note: 052 - Vector Storage

## Overview
Feature #52 implements in-memory contiguous vector storage in crate `hnsw-storage`:
- `VectorStore` trait defining `put`, `get`, `len`, `iter`, `memory_bytes`.
- `InMemoryVectorStore` implementation backed by a single contiguous `Vec<f32>` arena and an `id -> slot` index map.
- Validates records against `CollectionSchema` on `put`.
- Prevents duplicate insertion (`StorageError::DuplicateId`).
- Guarantees bit-exact floating point round-trip (verified via `f32::to_bits`) across normal, subnormal/denormal, extremes (`MIN_POSITIVE`, `MAX`), negative zero (`-0.0`), and near-zero values (`1e-38`).
- Thread-safe concurrency: uses `parking_lot::RwLock` for atomic writes and parallel non-blocking reads with 0 torn reads.

## Contiguous Arena Architecture
```
Arena: [ v0_0, v0_1, ..., v0_d-1 | v1_0, v1_1, ..., v1_d-1 | ... ]
       \_______ slot 0 ________/   \_______ slot 1 ________/

id_to_slot: VectorId -> slot_index (usize)
slot_metadata: slot_index -> SlotRecord { id, payload, version, deleted }
```
- Contiguous storage ensures cache locality and prepares for SIMD distance computations in subsequent features.
- Memory overhead: `memory_bytes()` calculates `arena.capacity() * sizeof(f32) + id_to_slot.capacity() * (sizeof(VectorId) + sizeof(usize)) + metadata`.

## Known Gaps
- **Durability**: Persistence, WAL append, fsync, and disk snapshots are explicitly deferred to features #25 and #82. `InMemoryVectorStore` is volatile in-memory storage.
- **Update/Replace**: Replacing existing IDs belongs to feature #74.
