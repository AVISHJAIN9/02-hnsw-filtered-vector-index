//! Bounded heap top-k candidate selection with deterministic tie-breaking.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use thiserror::Error;

use crate::response::ScoredPoint;

/// Errors arising from top-k selection.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum TopKError {
    #[error("candidate score is NaN")]
    NanScore,
}

/// Min-heap element wrapper where worse candidates have greater priority to be evicted first.
#[derive(Debug)]
pub struct MinHeapElement(pub ScoredPoint);

impl PartialEq for MinHeapElement {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for MinHeapElement {}

impl PartialOrd for MinHeapElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinHeapElement {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap pops the maximum element.
        // A is worse than B if A.score < B.score or (A.score == B.score && A.id > B.id).
        let ord = other.0.score.total_cmp(&self.0.score);
        if ord == Ordering::Equal {
            self.0.id.cmp(&other.0.id)
        } else {
            ord
        }
    }
}

/// Select the best `k` candidates from `candidates` using a bounded min-heap.
///
/// Guarantees:
/// - Exactly $\min(k, n)$ items returned, sorted best-first (highest score first, smaller `VectorId` on tie).
/// - $k = 0 \implies \emptyset$.
/// - $n = 0 \implies \emptyset$.
/// - Rejects NaN scores with `TopKError::NanScore`.
pub fn top_k(
    candidates: impl IntoIterator<Item = ScoredPoint>,
    k: usize,
) -> Result<Vec<ScoredPoint>, TopKError> {
    if k == 0 {
        for c in candidates {
            if c.score.is_nan() {
                return Err(TopKError::NanScore);
            }
        }
        return Ok(Vec::new());
    }

    let mut heap = BinaryHeap::with_capacity(k);

    for point in candidates {
        if point.score.is_nan() {
            return Err(TopKError::NanScore);
        }

        if heap.len() < k {
            heap.push(MinHeapElement(point));
        } else {
            let elem = MinHeapElement(point);
            let worst = heap.peek().expect("heap non-empty");
            if elem.cmp(worst) == Ordering::Less {
                heap.pop();
                heap.push(elem);
            }
        }
    }

    let mut results: Vec<ScoredPoint> = heap.into_iter().map(|e| e.0).collect();
    results.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));

    Ok(results)
}
