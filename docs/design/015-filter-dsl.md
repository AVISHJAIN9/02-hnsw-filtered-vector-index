# Design Document: 015 - Filter DSL

## 1. Overview
Feature #15 implements the typed Filter AST, JSON parser, canonicalization engine, evaluation semantics over VectorRecord payloads, and integration into `hnsw-service` and `hnsw-api`.

## 2. Public API & Types

### 2.1 AST Definition (`hnsw-filter::ast`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must: Vec<Clause>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub should: Vec<Clause>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must_not: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Clause {
    Condition(Condition),
    Nested(Filter),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    #[serde(flatten)]
    pub predicate: Predicate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Predicate {
    Match(MatchPredicate),
    Range(RangePredicate),
}
```

### 2.2 Error Types (`hnsw-filter::error::FilterError`)
- `UnknownField(String)`: Field not recognized in filter JSON.
- `TypeMismatch { key: String, expected: &'static str, actual: &'static str }`: E.g., range on string/bool.
- `MixedTypeList { key: String }`: `match.any` with heterogeneous types.
- `InvalidRangeBound { key: String, reason: &'static str }`: NaN/Inf/bool in range bounds.
- `ConflictingBounds { key: String, reason: &'static str }`: E.g., both `gt` and `gte`, or `lt` and `lte`.
- `MaxDepthExceeded { depth: usize, max: usize }`: Filter exceeds recursion limit.
- `MaxClausesExceeded { clauses: usize, max: usize }`: Filter exceeds clause budget limit.

### 2.3 Evaluation Semantics
- `must`: All clauses must evaluate to `true`. An empty `must` list matches.
- `should`: At least one clause must evaluate to `true`. An empty `should` list matches.
- `must_not`: None of the clauses may evaluate to `true`.
- `missing key`: Never matches `match` or `range`, which means `must_not` on a missing key matches `true`.
- Numeric comparisons: Integers and floats compare numerically (e.g., `1 == 1.0`).
- String sets: A payload containing `StringSet` matches if the set contains the target value or intersects with `any`.

### 2.4 Canonicalization
Deterministic canonical form:
1. Flatten single-clause nested groups where logical equivalence is preserved.
2. Stable clause ordering (sorted canonically).
3. Stable key ordering in serializations.
4. Identical ASTs produce byte-identical JSON.

## 3. Testing Strategy
- Tests first before implementation:
  - AST parsing & validation tests (`backend/crates/filter/tests/parser_tests.rs`).
  - Canonicalization & proptest round-trip tests (`backend/crates/filter/tests/canonical_tests.rs`).
  - Truth-table evaluator tests for all combinations (`backend/crates/filter/tests/evaluator_tests.rs`).
  - API integration tests (`backend/crates/api/tests/search_api_contract_tests.rs`).
