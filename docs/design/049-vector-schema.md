# Design Document: Feature #49 "Vector schema"

## 1. Feature Overview & Metadata

- **Feature Number**: 49
- **Feature Name**: Vector schema
- **Workstream**: Database / Common Core
- **Description**: Define vector ID, dimension, metric, payload and version model
- **Priority**: P0
- **Dependencies**: None (`-`)
- **Acceptance Criteria**: Schema supports required lifecycle
- **KPI / Target**: Schema supports required lifecycle (100% passing tests for schema validation, serde, lifecycle state transitions, and rejection of older schema versions)

---

## 2. Decisions & Architectural Invariants

- **Language & Toolchain**: Rust stable pinned via `rust-toolchain.toml`. `Cargo.lock` committed and builds use `--locked`.
- **Dependencies**: `serde` (with `derive`), `serde_json`, `thiserror`.
- **Vectors**: Single-precision `f32`.
- **Metrics**: `L2`, `Cosine`, `InnerProduct`. Cosine metric strictly rejects an all-zero vector (since zero vectors cannot be normalized and have undefined cosine similarity).
- **CollectionSchema**: `{ schema_version: SchemaVersion, dimension: Dimension, metric: Metric }`. `VectorRecord::new` and `VectorRecord::update` validate vector inputs against a referenced `CollectionSchema`.
- **Deserialization Invariant (TryFrom)**: Deserialization of `VectorRecord` does not deserialize directly into unvalidated fields; it deserializes into a private `RawVectorRecord` and validates via `TryFrom<RawVectorRecord> for VectorRecord`, guaranteeing that corrupted or malformed JSON (such as vector length mismatch or non-finite numbers) is rejected on load.
- **Payload & StringSet**:
  - `Payload`: Type alias / newtype over `BTreeMap<String, PayloadValue>`.
  - `StringSet`: `BTreeSet<String>`.
  - `PayloadValue` JSON serialization:
    - Integer number -> `PayloadValue::Int64(i64)`.
    - Floating point number -> `PayloadValue::Float64(f64)`.
    - Distinct representation: `1` deserializes to `Int64(1)`, and `1.0` deserializes to `Float64(1.0)`.
    - `Float64` payload values must be finite (reject `NaN`, `+Inf`, `-Inf`).
- **Configuration Limits**:
  - Centralized in `backend/crates/common/src/config.rs`.
  - Overridable via environment variables with the `HNSW_` prefix:
    - `HNSW_MAX_DIMENSION` (default: `4096` — covers standard embedding models like OpenAI text-embedding-3-large 3072d with safety headroom).
    - `HNSW_MAX_K` (default: `1000` — bounds candidate heap allocation and prevents server OOM).
    - `HNSW_MAX_FILTER_DEPTH` (default: `10` — prevents recursive stack overflow on malicious nested AST filters).
    - `HNSW_MAX_FILTER_CLAUSES` (default: `100` — bounds filter evaluation CPU budget per query).
    - `HNSW_MAX_BODY_BYTES` (default: `10485760` / 10MB — prevents memory exhaustion from oversized HTTP payloads).
  - Env-var parsing tests must be serialized to prevent race conditions on process environment.
- **Lifecycle & State Transitions**:
  - `VectorId`: Strongly-typed newtype over `u64`. String and UUID identifiers are intentionally unsupported in this core schema.
  - Initial creation: `VectorRecord::new` sets `version = 1`, `deleted = false`.
  - Update: `record.update(schema, vector, payload)` validates against schema, replaces vector and payload, increments `version` using `checked_add` (returns `SchemaError::VersionOverflow` on overflow), and clears `deleted = false` (updating after tombstone revives the record).
  - Tombstone: `record.tombstone()` sets `deleted = true` and increments `version` using `checked_add`. Calling `tombstone()` on an already-deleted record returns `Err(SchemaError::AlreadyDeleted)` with no version bump.
- **Schema Versioning**:
  - Supported version is `SchemaVersion::V1 = 1`.
  - Deserializing JSON with older versions (e.g. `0`) or newer/unsupported versions (e.g. `2`) must return `SchemaError::UnsupportedVersion`.
- **Known Gaps**:
  - No payload size limit is enforced in #49; documented as a known gap to be addressed in feature #16 (Query validation).

---

## 3. Files to Create and Modify

1. **`rust-toolchain.toml`** (New):
   - Pin Rust stable channel: `[toolchain]\nchannel = "stable"`.
2. **`.env.example`** (Update):
   - Document `HNSW_MAX_DIMENSION`, `HNSW_MAX_K`, `HNSW_MAX_FILTER_DEPTH`, `HNSW_MAX_FILTER_CLAUSES`, `HNSW_MAX_BODY_BYTES`.
3. **`backend/crates/common/Cargo.toml`** (Update):
   - Add dependencies: `serde = { version = "1.0", features = ["derive"] }`, `serde_json = "1.0"`, `thiserror = "1.0"`.
   - Add dev-dependencies: `serial_test = "3.1"`.
4. **`backend/crates/common/src/config.rs`** (New):
   - Centralized `Config` struct with documented defaults and reasons, overridable by `HNSW_*` environment variables.
5. **`backend/crates/common/src/error.rs`** (New):
   - Typed `SchemaError` variants: `DimensionZero`, `DimensionTooLarge`, `DimensionMismatch`, `NonFiniteFloat`, `ZeroVectorForCosine`, `UnsupportedVersion`, `AlreadyDeleted`, `VersionOverflow`.
6. **`backend/crates/common/src/schema.rs`** (New):
   - Types: `VectorId`, `Dimension`, `Metric`, `PayloadValue`, `Payload`, `SchemaVersion`, `CollectionSchema`, `VectorRecord`.
   - Custom serde visitor / deserializer for `PayloadValue` handling integers vs floats and finiteness.
   - `TryFrom<RawVectorRecord>` for `VectorRecord`.
   - Record lifecycle methods: `new`, `update`, `tombstone`.
7. **`backend/crates/common/src/lib.rs`** (Update):
   - Export `config`, `error`, `schema`.
8. **`backend/crates/common/tests/golden/vector_record_v1.json`** (New):
   - Golden JSON fixture representing a valid serialized V1 vector record.
9. **`backend/crates/common/tests/schema_lifecycle_tests.rs`** (New):
   - Comprehensive test suite covering criteria a through e and all requested test cases.

---

## 4. Public Types and APIs

```rust
// backend/crates/common/src/schema.rs

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::error::SchemaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VectorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dimension(u32);

impl Dimension {
    pub fn new(dim: u32, max_dimension: u32) -> Result<Self, SchemaError>;
    pub fn get(&self) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Metric {
    L2,
    Cosine,
    InnerProduct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadValue {
    String(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
    StringSet(BTreeSet<String>),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Payload(pub BTreeMap<String, PayloadValue>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum SchemaVersion {
    #[serde(rename = "1")]
    V1 = 1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub schema_version: SchemaVersion,
    pub dimension: Dimension,
    pub metric: Metric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawVectorRecord")]
pub struct VectorRecord {
    pub schema_version: SchemaVersion,
    pub id: VectorId,
    pub vector: Vec<f32>,
    pub payload: Payload,
    pub version: u64,
    pub deleted: bool,
}

impl VectorRecord {
    pub fn new(
        schema: &CollectionSchema,
        id: VectorId,
        vector: Vec<f32>,
        payload: Payload,
    ) -> Result<Self, SchemaError>;

    pub fn update(
        &mut self,
        schema: &CollectionSchema,
        vector: Vec<f32>,
        payload: Payload,
    ) -> Result<(), SchemaError>;

    pub fn tombstone(&mut self) -> Result<(), SchemaError>;
}
```

---

## 5. Test List Mapped 1:1 to Acceptance Criteria

### Criterion a: Types and definitions
- **`test_types_compilation_and_instantiation`**:
  - Compiles and verifies types: `VectorId`, `Dimension`, `Metric`, `PayloadValue`, `Payload`, `VectorRecord`, `SchemaVersion`, `CollectionSchema`.

### Criterion b: Serde round-trip, distinct 1 vs 1.0, and golden JSON
- **`test_serde_roundtrip_all_types`**:
  - Serde symmetry test for all types.
- **`test_payload_value_integer_vs_float_roundtrip`**:
  - Proves `1` deserializes to `PayloadValue::Int64(1)` and `1.0` deserializes to `PayloadValue::Float64(1.0)`.
  - Rejects non-finite floats in payload (`NaN`, `+Inf`, `-Inf`).
- **`test_golden_json_file`**:
  - Compares serialization against `tests/golden/vector_record_v1.json`.

### Criterion c: Construction validation
- **`test_construct_rejects_dimension_zero`**:
  - Dimension 0 returns `Err(SchemaError::DimensionZero)`.
- **`test_construct_rejects_dimension_exceeding_max`**:
  - Dimension > max returns `Err(SchemaError::DimensionTooLarge)`.
- **`test_construct_rejects_vector_len_mismatch`**:
  - Vector length != schema dimension returns `Err(SchemaError::DimensionMismatch)`.
- **`test_construct_rejects_non_finite_floats`**:
  - Floats with `NaN` or `Inf` return `Err(SchemaError::NonFiniteFloat)`.
- **`test_construct_rejects_zero_vector_for_cosine`**:
  - All-zero vector for `Metric::Cosine` returns `Err(SchemaError::ZeroVectorForCosine)`.
- **`test_deserialization_rejects_wrong_vector_length`**:
  - Loading JSON with mismatched vector length via `serde_json::from_str` is rejected (using `RawVectorRecord` + `TryFrom`).

### Criterion d: Lifecycle modeling and testing
- **`test_lifecycle_new_record_initial_state`**:
  - New record has `version == 1` and `deleted == false`.
- **`test_lifecycle_update_increments_version`**:
  - `.update(...)` increments version (`1 -> 2`), updates vector/payload, `deleted` is `false`.
- **`test_lifecycle_tombstone_marks_deleted`**:
  - `.tombstone()` sets `deleted = true` and increments version (`2 -> 3`).
- **`test_lifecycle_tombstone_already_deleted_fails`**:
  - Calling `.tombstone()` on an already-deleted record returns `Err(SchemaError::AlreadyDeleted)` and does NOT bump version.
- **`test_lifecycle_update_after_tombstone_clears_deleted`**:
  - Calling `.update(...)` on a tombstoned record resets `deleted = false` and increments version (`3 -> 4`).

### Criterion e: Schema version rejection
- **`test_older_schema_version_rejected`**:
  - Deserializing JSON containing `schema_version: 0` and `schema_version: 2` returns `SchemaError::UnsupportedVersion`.
