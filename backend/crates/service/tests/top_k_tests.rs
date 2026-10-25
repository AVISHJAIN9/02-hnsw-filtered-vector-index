use hnsw_common::schema::VectorId;
use hnsw_service::response::ScoredPoint;
use hnsw_service::top_k::{top_k, TopKError};
use proptest::prelude::*;

fn point(id: u64, score: f32) -> ScoredPoint {
    ScoredPoint {
        id: VectorId(id),
        score,
        payload: None,
    }
}

#[test]
fn test_top_k_empty_inputs() {
    // k = 0 returns empty
    let candidates = vec![point(1, 0.9), point(2, 0.8)];
    let res = top_k(candidates, 0).unwrap();
    assert!(res.is_empty());

    // n = 0 returns empty
    let empty_candidates: Vec<ScoredPoint> = vec![];
    let res = top_k(empty_candidates, 5).unwrap();
    assert!(res.is_empty());
}

#[test]
fn test_top_k_exact_min_k_n_and_sorted_best_first() {
    let candidates = vec![
        point(1, 0.5),
        point(2, 0.9),
        point(3, 0.1),
        point(4, 0.8),
        point(5, 0.3),
    ];

    // k < n: returns exactly min(3, 5) = 3 items sorted best-first
    let res = top_k(candidates.clone(), 3).unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].id, VectorId(2));
    assert_eq!(res[0].score, 0.9);
    assert_eq!(res[1].id, VectorId(4));
    assert_eq!(res[1].score, 0.8);
    assert_eq!(res[2].id, VectorId(1));
    assert_eq!(res[2].score, 0.5);

    // k > n: returns exactly min(10, 5) = 5 items
    let res_all = top_k(candidates, 10).unwrap();
    assert_eq!(res_all.len(), 5);
    assert_eq!(res_all[0].id, VectorId(2));
    assert_eq!(res_all[4].id, VectorId(3));
}

#[test]
fn test_top_k_deterministic_tie_breaking_by_vector_id() {
    // Multiple candidates with identical scores
    let candidates = vec![
        point(10, 0.7),
        point(2, 0.7),
        point(5, 0.7),
        point(1, 0.9),
        point(8, 0.7),
    ];

    // Top 3 should be: point 1 (0.9), then point 2 (0.7), point 5 (0.7)
    let res = top_k(candidates, 3).unwrap();
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].id, VectorId(1));
    assert_eq!(res[1].id, VectorId(2));
    assert_eq!(res[2].id, VectorId(5));
}

#[test]
fn test_top_k_rejects_nan_score() {
    let candidates = vec![point(1, 0.5), point(2, f32::NAN), point(3, 0.8)];
    let err = top_k(candidates, 2).unwrap_err();
    assert_eq!(err, TopKError::NanScore);

    // Also when k = 0
    let candidates_k0 = vec![point(1, f32::NAN)];
    let err_k0 = top_k(candidates_k0, 0).unwrap_err();
    assert_eq!(err_k0, TopKError::NanScore);

    // Verify error format
    assert_eq!(format!("{}", TopKError::NanScore), "candidate score is NaN");
    let svc_err: hnsw_service::error::ServiceError = TopKError::NanScore.into();
    assert_eq!(
        format!("{}", svc_err),
        "top-k error: candidate score is NaN"
    );
}

#[test]
fn test_min_heap_element_equality() {
    use hnsw_service::top_k::MinHeapElement;
    let elem1 = MinHeapElement(point(1, 0.5));
    let elem2 = MinHeapElement(point(1, 0.5));
    let elem3 = MinHeapElement(point(2, 0.5));
    assert_eq!(elem1, elem2);
    assert_ne!(elem1, elem3);
}

proptest! {
    #[test]
    fn test_proptest_top_k_matches_sort_then_truncate(
        scores in proptest::collection::vec(-100.0f32..100.0f32, 0..100),
        k in 0usize..50,
    ) {
        let candidates: Vec<ScoredPoint> = scores
            .iter()
            .enumerate()
            .map(|(idx, &s)| point((idx + 1) as u64, s))
            .collect();

        // 1. Reference: Sort-then-truncate
        let mut reference = candidates.clone();
        reference.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap()
                .then_with(|| a.id.cmp(&b.id))
        });
        reference.truncate(k);

        // 2. Implementation: top_k bounded heap
        let actual = top_k(candidates, k).unwrap();

        prop_assert_eq!(actual.len(), reference.len());
        for (act, exp) in actual.iter().zip(reference.iter()) {
            prop_assert_eq!(act.id, exp.id);
            prop_assert_eq!(act.score, exp.score);
        }
    }
}
