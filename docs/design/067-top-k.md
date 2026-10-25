# Design Note: 067 - Top-K Selection

## Overview
Feature #67 implements exact bounded heap top-$k$ candidate selection in `hnsw-service`:
- Selects the best $k$ of $n$ scored candidates (`ScoredPoint`) using a bounded min-heap of size at most $k$.
- Time complexity: $O(n \log k)$, auxiliary space: $O(k)$.
- Deterministic tie-breaking: Candidates with identical scores are ordered by `VectorId` ascending (smaller ID ranks ahead).
- NaN rejection: Candidates containing NaN scores are rejected with `TopKError::NanScore`.
- Edge cases: $k = 0 \implies \emptyset$, $n = 0 \implies \emptyset$, always returns exactly $\min(k, n)$ items sorted best-first.

## API Specification
```rust
#[derive(Debug, Error, PartialEq, Clone)]
pub enum TopKError {
    #[error("candidate score is NaN")]
    NanScore,
}

pub fn top_k(
    candidates: impl IntoIterator<Item = ScoredPoint>,
    k: usize,
) -> Result<Vec<ScoredPoint>, TopKError>;
```

## Heap Invariants & Ordering
To retain the $k$ best elements with best-first output (highest score first, tie-break by smaller `VectorId`):
- A bounded min-heap holds at most $k$ elements.
- The heap's root (minimum element) represents the "worst" candidate among the current top $k$.
- Comparison: candidate $A$ is worse than $B$ if $A.\text{score} < B.\text{score}$ or ($A.\text{score} == B.\text{score}$ and $A.\text{id} > B.\text{id}$).
- After scanning all $n$ candidates, elements are extracted and sorted in descending order of preference.

## Integration & Property Testing
- Integrated into `ResponseBuilder::build`.
- Proptest verifies $O(n \log k)$ heap selection produces identical results to full $O(n \log n)$ sort-and-truncate for arbitrary vectors and $k$.
- All existing #13 search service tests continue to pass.
