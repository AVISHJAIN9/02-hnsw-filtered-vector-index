use hnsw_common::schema::Metric;
use hnsw_simd::error::MetricError;
use hnsw_simd::{cosine_distance, distance, inner_product_distance, l2_distance};
use proptest::prelude::*;
use serde::Deserialize;

#[test]
fn test_hand_computed_l2() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 6.0, 8.0];
    // (4-1)^2 + (6-2)^2 + (8-3)^2 = 9 + 16 + 25 = 50.0
    let dist = l2_distance(&a, &b).unwrap();
    assert_eq!(dist, 50.0);

    let dist_dispatch = distance(Metric::L2, &a, &b).unwrap();
    assert_eq!(dist_dispatch, 50.0);
}

#[test]
fn test_hand_computed_cosine() {
    let a = [1.0, 0.0];
    let b = [0.0, 2.0];
    // Orthogonal: dot = 0 => cos_dist = 1.0 - 0.0 = 1.0
    assert_eq!(cosine_distance(&a, &b).unwrap(), 1.0);

    let c = [3.0, 0.0];
    // Same direction: dot = 3, norm_a = 1, norm_c = 3 => cos_dist = 1.0 - 1.0 = 0.0
    assert!((cosine_distance(&a, &c).unwrap()).abs() < 1e-6);

    let d = [-2.0, 0.0];
    // Opposite direction: cos_dist = 1.0 - (-1.0) = 2.0
    assert!((cosine_distance(&a, &d).unwrap() - 2.0).abs() < 1e-6);

    let dist_dispatch = distance(Metric::Cosine, &a, &b).unwrap();
    assert_eq!(dist_dispatch, 1.0);
}

#[test]
fn test_hand_computed_inner_product() {
    let a = [2.0, 3.0, -1.0];
    let b = [4.0, -1.0, 2.0];
    // dot = 2*4 + 3*(-1) + (-1)*2 = 8 - 3 - 2 = 3.0
    // IP distance = -dot = -3.0 (negative distance!)
    assert_eq!(inner_product_distance(&a, &b).unwrap(), -3.0);

    let dist_dispatch = distance(Metric::InnerProduct, &a, &b).unwrap();
    assert_eq!(dist_dispatch, -3.0);
}

#[test]
fn test_dimension_mismatch_error() {
    let a = [1.0, 2.0];
    let b = [1.0, 2.0, 3.0];

    assert_eq!(
        l2_distance(&a, &b).unwrap_err(),
        MetricError::DimensionMismatch {
            expected: 2,
            actual: 3
        }
    );
    assert_eq!(
        cosine_distance(&a, &b).unwrap_err(),
        MetricError::DimensionMismatch {
            expected: 2,
            actual: 3
        }
    );
    assert_eq!(
        inner_product_distance(&a, &b).unwrap_err(),
        MetricError::DimensionMismatch {
            expected: 2,
            actual: 3
        }
    );
}

#[test]
fn test_cosine_zero_vector_error() {
    let a = [0.0, 0.0, 0.0];
    let b = [1.0, 2.0, 3.0];

    assert_eq!(
        cosine_distance(&a, &b).unwrap_err(),
        MetricError::ZeroVector
    );
    assert_eq!(
        cosine_distance(&b, &a).unwrap_err(),
        MetricError::ZeroVector
    );
    assert_eq!(
        cosine_distance(&a, &a).unwrap_err(),
        MetricError::ZeroVector
    );
}

#[test]
fn test_properties_identity_and_non_negativity() {
    let a = [1.5, -2.5, 3.0];
    let b = [0.5, 4.0, -1.0];

    // L2: Identity D(a, a) = 0, Non-negativity D(a, b) >= 0
    assert_eq!(l2_distance(&a, &a).unwrap(), 0.0);
    assert!(l2_distance(&a, &b).unwrap() >= 0.0);

    // Cosine: Identity D(a, a) = 0, Non-negativity D(a, b) >= 0
    assert!(cosine_distance(&a, &a).unwrap().abs() < 1e-6);
    assert!(cosine_distance(&a, &b).unwrap() >= 0.0);

    // InnerProduct: Can be negative!
    let pos_a = [2.0, 3.0];
    let pos_b = [4.0, 5.0];
    assert!(inner_product_distance(&pos_a, &pos_b).unwrap() < 0.0);
}

#[derive(Deserialize)]
struct GoldenCase {
    id: usize,
    dimension: usize,
    a: Vec<f32>,
    b: Vec<f32>,
    expected: GoldenExpected,
}

#[derive(Deserialize)]
struct GoldenExpected {
    l2: f32,
    cosine: f32,
    inner_product: f32,
}

#[test]
fn test_golden_cross_check_20_cases() {
    let raw = include_str!("golden/metrics.json");
    let cases: Vec<GoldenCase> = serde_json::from_str(raw).expect("valid golden metrics JSON");
    assert_eq!(cases.len(), 20, "Must have exactly 20 golden cases");

    for c in cases {
        assert_eq!(c.a.len(), c.dimension);
        assert_eq!(c.b.len(), c.dimension);

        let l2 = l2_distance(&c.a, &c.b).unwrap();
        let cos = cosine_distance(&c.a, &c.b).unwrap();
        let ip = inner_product_distance(&c.a, &c.b).unwrap();

        let diff_l2 = (l2 - c.expected.l2).abs();
        let diff_cos = (cos - c.expected.cosine).abs();
        let diff_ip = (ip - c.expected.inner_product).abs();

        assert!(
            diff_l2 <= 1e-5,
            "case {}: l2 mismatch: actual {}, expected {}, diff {}",
            c.id,
            l2,
            c.expected.l2,
            diff_l2
        );
        assert!(
            diff_cos <= 1e-5,
            "case {}: cosine mismatch: actual {}, expected {}, diff {}",
            c.id,
            cos,
            c.expected.cosine,
            diff_cos
        );
        assert!(
            diff_ip <= 1e-5,
            "case {}: inner_product mismatch: actual {}, expected {}, diff {}",
            c.id,
            ip,
            c.expected.inner_product,
            diff_ip
        );
    }
}

proptest! {
    #[test]
    fn test_proptest_symmetry_and_l2_identity(
        v1 in proptest::collection::vec(-10.0f32..10.0f32, 1..32),
        v2 in proptest::collection::vec(-10.0f32..10.0f32, 1..32),
    ) {
        // Equal dimensions for valid distance computation
        let min_len = v1.len().min(v2.len());
        let a = &v1[..min_len];
        let b = &v2[..min_len];

        // L2 Identity D(a, a) == 0.0
        let l2_self = l2_distance(a, a).unwrap();
        prop_assert!(l2_self.abs() < 1e-5, "L2(a, a) must be 0: got {}", l2_self);

        // L2 Symmetry D(a, b) == D(b, a)
        let l2_ab = l2_distance(a, b).unwrap();
        let l2_ba = l2_distance(b, a).unwrap();
        prop_assert!((l2_ab - l2_ba).abs() < 1e-5);

        // IP Symmetry D(a, b) == D(b, a)
        let ip_ab = inner_product_distance(a, b).unwrap();
        let ip_ba = inner_product_distance(b, a).unwrap();
        prop_assert!((ip_ab - ip_ba).abs() < 1e-5);

        // Cosine Symmetry D(a, b) == D(b, a) (if non-zero)
        let a_norm_sq: f32 = a.iter().map(|x| x * x).sum();
        let b_norm_sq: f32 = b.iter().map(|x| x * x).sum();
        if a_norm_sq > 1e-6 && b_norm_sq > 1e-6 {
            let cos_ab = cosine_distance(a, b).unwrap();
            let cos_ba = cosine_distance(b, a).unwrap();
            prop_assert!((cos_ab - cos_ba).abs() < 1e-5);
        }
    }
}

#[test]
fn test_non_finite_floats_rejected() {
    let nan_vec = [1.0, f32::NAN];
    let inf_vec = [f32::INFINITY, 2.0];
    let normal = [1.0, 2.0];

    assert_eq!(
        l2_distance(&nan_vec, &normal).unwrap_err(),
        MetricError::NonFiniteFloat
    );
    assert_eq!(
        cosine_distance(&normal, &inf_vec).unwrap_err(),
        MetricError::NonFiniteFloat
    );
    assert_eq!(
        inner_product_distance(&inf_vec, &normal).unwrap_err(),
        MetricError::NonFiniteFloat
    );
}
