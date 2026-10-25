# Design Document: Feature #14 "Search API"

## 1. Feature Overview & Metadata

- **Feature Number**: 14
- **Feature Name**: Search API
- **Workstream**: Backend / API Ingress
- **Description**: Versioned endpoint for vector search + filters + top-k
- **Priority**: P0
- **Dependencies**: Search service (#13)
- **Acceptance Criteria**: Contract and validation tests pass
- **KPI / Target**: Contract and validation tests pass (100% contract compliance, all HTTP error cases tested, zero business logic in handlers, graceful shutdown verified)

---

## 2. Decisions & Architectural Invariants

- **Crate**: `backend/crates/api` (`hnsw-api`), binary `backend/src/main.rs`.
- **Dependencies**: `hnsw-service`, `hnsw-common`, `axum`, `tokio`, `serde`, `serde_json`, `tracing`; dev-dependencies: `tower`, `http-body-util`.
- **Endpoint**: `POST /v1/search`
  - Request body: `{ "vector": [f32], "k": int, "filter"?: object, "include_payload"?: bool }`
  - Rejects unknown fields (`#[serde(deny_unknown_fields)]`) with HTTP 400.
  - If `"filter"` is present:
    - Must be a JSON object (if not an object, return HTTP 400).
    - In #14, filters are not yet supported until #15, so a present filter object returns HTTP 501 with code `"filter_not_supported"`. It must never be silently ignored.
  - Response: `{ "results": [{ "id": u64, "score": f32, "payload"?: object }], "took_ms": f64 }`.
- **Error Handling**:
  - Malformed JSON -> 400 (`malformed_json`).
  - Unknown fields -> 400 (`unknown_field`).
  - Wrong Content-Type -> 415 (`unsupported_media_type`).
  - Request body exceeding `Config.max_body_bytes` -> 413 (`payload_too_large`).
  - Unknown route -> 404 JSON (`not_found`).
  - Wrong method -> 405 JSON (`method_not_allowed`).
  - Uniform envelope: `{"error":{"code":"...","message":"..."}}`.
- **Zero Business Logic in Handlers**:
  - The handler simply deserializes, validates transport concerns, invokes `SearchService::search`, and formats the response.
- **Server Lifecycle**:
  - Expose `serve(listener: tokio::net::TcpListener, shutdown: impl Future<Output = ()> + Send + 'static)`.
  - `main.rs` configures SIGTERM / Ctrl-C graceful shutdown.
  - Binary starts with `EmptyIndexRetriever` (retriever returning no candidates for zero vectors).

---

## 3. Files to Create and Modify

1. **`backend/crates/api/Cargo.toml`**:
   - Add dependencies: `hnsw-service`, `hnsw-common`, `axum = "0.7"`, `tokio = { version = "1.38", features = ["full"] }`, `serde`, `serde_json`, `tracing`.
   - Add dev-dependencies: `tower = { version = "0.4", features = ["util"] }`, `http-body-util = "0.1"`.
2. **`backend/crates/api/src/error.rs`**:
   - `ApiError` enum and Axum `IntoResponse` implementation producing uniform JSON error format.
3. **`backend/crates/api/src/dto.rs`**:
   - `SearchRequestDto` with `#[serde(deny_unknown_fields)]` and `SearchResponseDto`.
4. **`backend/crates/api/src/handlers.rs`**:
   - `search_handler` delegating directly to `SearchService`.
5. **`backend/crates/api/src/server.rs`**:
   - Router creation with body size limit middleware, 404/405 fallback handlers, and `serve(listener, shutdown)`.
6. **`backend/crates/api/src/lib.rs`**:
   - Re-exports.
7. **`backend/src/main.rs`**:
   - Executable entrypoint with `EmptyIndexRetriever` and graceful shutdown signal handler.
8. **`backend/crates/api/tests/golden/search_request_valid.json` & `search_response_valid.json`**:
   - Golden contract fixtures.
9. **`backend/crates/api/tests/search_api_contract_tests.rs`**:
   - In-process contract and validation tests using `tower::ServiceExt::oneshot`.

---

## 4. Test List Mapped 1:1 to Criteria

- **Criterion a & b**: `test_search_api_contracts_and_rejections`
  - Successful search contract (`results`, `took_ms`).
  - Rejection of unknown fields (400).
  - Rejection of non-object filter (400).
  - Rejection of present filter object (501 `filter_not_supported`).
  - Rejection of malformed JSON (400).
  - Rejection of wrong Content-Type (415).
  - Rejection of oversized body (413).
  - Rejection of unknown route (404 JSON).
  - Rejection of wrong HTTP method (405 JSON).
- **Criterion c**: `test_golden_contract_files`
  - In-process test comparing request/response against `tests/golden/`.
- **Criterion d**: `test_handler_has_no_business_logic`
  - Uses a spy retriever: invalid vector yields ServiceError without calling retriever; valid request calls retriever exactly once.
- **Criterion e**: `test_serve_graceful_shutdown`
  - Binds to `127.0.0.1:0`, performs request, triggers shutdown future, asserts graceful termination within 2 seconds.
