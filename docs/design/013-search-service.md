# Design Document: Feature #13 "Search service"

## 1. Feature Overview & Metadata

- **Feature Number**: 13
- **Feature Name**: Search service
- **Workstream**: Backend / Service Core
- **Description**: Define query, filter, retrieval and response service boundaries
- **Priority**: P0
- **Dependencies**: None (`-`)
- **Acceptance Criteria**: Responsibilities are independently testable
- **KPI / Target**: Responsibilities are independently testable (5/5 component unit test suites running without HTTP server or index; fake retriever driving end-to-end flow; zero axum/hyper/tower dependencies)

---

## 2. Architectural Boundaries & Decisions

- **Crate**: `backend/crates/service` (`hnsw-service`). Depends solely on `hnsw-common` and `thiserror`.
- **Zero HTTP Dependencies**: Must NOT depend on `axum`, `hyper`, `tower`, or `tokio`'s HTTP stack. Enforced via `cargo tree -p hnsw-service`.
- **Type Reuse**: Reuses types from `hnsw-common` (`VectorId`, `VectorRecord`, `CollectionSchema`, `Dimension`, `Metric`, `Payload`, `Config`, `SchemaError`).
- **Boundaries & Components**:
  1. **`Validator`**: Validates `SearchRequest` bounds against `CollectionSchema` and `Config` (k in `1..=max_k`, dimension matching, finite floats, non-zero for Cosine).
  2. **`FilterEvaluator`**: Trait defining predicate evaluation over `VectorRecord`. Provides a real `NoFilter` implementation returning `Ok(true)` for all records. (Full AST filtering deferred to feature #15).
  3. **`Retriever`**: Trait defining candidate retrieval: `retrieve(&self, vector: &[f32], k: usize) -> Result<Vec<VectorRecord>, ServiceError>`. Real HNSW retriever arrives in Gate G2; test-only fake retriever drives tests.
  4. **`ResponseBuilder`**: Calibrates distances/similarities to scores, enforces top-k truncation, handles optional payload inclusion, and sets timing diagnostics (`took_ms`).
  5. **`SearchService`**: Composes the above components into `search(&self, req: SearchRequest) -> Result<SearchResponse, ServiceError>`.
  6. **`ServiceError`**: Strongly-typed enum leveraging `thiserror`, re-exporting `SchemaError` via `#[from]`.

---

## 3. Files to Create and Modify

1. **`backend/Cargo.toml`** (Update):
   - Add `"crates/service"` to workspace members.
2. **`backend/crates/service/Cargo.toml`** (New):
   - Manifest for `hnsw-service` depending on `hnsw-common` and `thiserror`.
3. **`backend/crates/service/src/error.rs`** (New):
   - `ServiceError` enum.
4. **`backend/crates/service/src/validator.rs`** (New):
   - `Validator` struct and validation logic.
5. **`backend/crates/service/src/filter.rs`** (New):
   - `FilterEvaluator` trait and `NoFilter` real implementation.
6. **`backend/crates/service/src/retriever.rs`** (New):
   - `Retriever` trait.
7. **`backend/crates/service/src/response.rs`** (New):
   - `ScoredPoint`, `SearchResponse`, `ResponseBuilder`.
8. **`backend/crates/service/src/service.rs`** (New):
   - `SearchRequest`, `SearchService<R, F>`.
9. **`backend/crates/service/src/lib.rs`** (New):
   - Module exports.
10. **`backend/crates/service/tests/search_service_tests.rs`** (New):
   - Independent unit and integration tests mapped 1:1 to criteria a–f.

---

## 4. Public Types and APIs

```rust
// backend/crates/service/src/error.rs
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ServiceError {
    #[error("schema error: {0}")]
    Schema(#[from] hnsw_common::error::SchemaError),

    #[error("invalid k: k must be greater than 0 and at most {max_k}, got {k}")]
    InvalidK { k: usize, max_k: usize },

    #[error("retrieval error: {0}")]
    Retrieval(String),

    #[error("filter error: {0}")]
    Filter(String),
}

// backend/crates/service/src/service.rs
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    pub k: usize,
    pub filter: Option<serde_json::Value>,
    pub include_payload: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredPoint {
    pub id: hnsw_common::schema::VectorId,
    pub score: f32,
    pub payload: Option<hnsw_common::schema::Payload>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<ScoredPoint>,
    pub took_ms: f64,
}

pub trait FilterEvaluator: Send + Sync {
    fn matches(&self, record: &hnsw_common::schema::VectorRecord) -> Result<bool, ServiceError>;
}

pub trait Retriever: Send + Sync {
    fn retrieve(&self, vector: &[f32], k: usize) -> Result<Vec<hnsw_common::schema::VectorRecord>, ServiceError>;
}

pub struct SearchService<R: Retriever, F: FilterEvaluator> {
    schema: hnsw_common::schema::CollectionSchema,
    config: hnsw_common::config::Config,
    validator: Validator,
    retriever: R,
    filter_evaluator: F,
    response_builder: ResponseBuilder,
}
```

---

## 5. Test List Mapped 1:1 to Criteria

- **Criterion a**: `test_search_service_composition_and_api`
  - Verifies `SearchService::search` contract composing all components.
- **Criterion b**:
  - `test_validator_unit_independent`: tests Validator with no HTTP and no index.
  - `test_filter_evaluator_unit_independent`: tests NoFilter with no HTTP and no index.
  - `test_response_builder_unit_independent`: tests ResponseBuilder distance-to-score calculation and truncation with no HTTP and no index.
- **Criterion c**: Checked via `cargo tree -p hnsw-service` proving absence of axum/hyper/tower.
- **Criterion d**: `test_fake_retriever_drives_end_to_end_flow`
  - Defines a `FakeRetriever` in test code only, exercising: validation -> retrieval -> filtering -> response building.
- **Criterion e**: `test_no_filter_evaluator_matches_all`
  - Verifies `NoFilter` evaluates `matches()` to `Ok(true)` for both normal and tombstoned records.
- **Criterion f**: `test_service_error_mapping`
  - Verifies `ServiceError` variants and automatic conversion from `SchemaError`.
