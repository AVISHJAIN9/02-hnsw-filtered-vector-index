use hnsw_common::schema::Metric;

use crate::error::MetricError;

fn validate_vectors(a: &[f32], b: &[f32]) -> Result<(), MetricError> {
    if a.len() != b.len() {
        return Err(MetricError::DimensionMismatch {
            expected: a.len(),
            actual: b.len(),
        });
    }

    for &val in a.iter().chain(b.iter()) {
        if !val.is_finite() {
            return Err(MetricError::NonFiniteFloat);
        }
    }

    Ok(())
}

/// Compute squared Euclidean distance (L2) between vectors `a` and `b`.
///
/// Formula: $\sum_{i=0}^{d-1} (a_i - b_i)^2$
pub fn l2_distance(a: &[f32], b: &[f32]) -> Result<f32, MetricError> {
    validate_vectors(a, b)?;

    let mut sum = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    Ok(sum)
}

/// Compute cosine distance between vectors `a` and `b`.
///
/// Formula: $1.0 - \frac{\sum a_i b_i}{\sqrt{\sum a_i^2} \sqrt{\sum b_i^2}}$
///
/// Returns `MetricError::ZeroVector` if either vector has a zero norm.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, MetricError> {
    validate_vectors(a, b)?;

    let mut dot = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    if norm_a_sq == 0.0 || norm_b_sq == 0.0 {
        return Err(MetricError::ZeroVector);
    }

    let denom = norm_a_sq.sqrt() * norm_b_sq.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    Ok(1.0 - sim)
}

/// Compute inner product distance between vectors `a` and `b`.
///
/// Formula: $-\sum_{i=0}^{d-1} a_i b_i = -\langle a, b \rangle$
pub fn inner_product_distance(a: &[f32], b: &[f32]) -> Result<f32, MetricError> {
    validate_vectors(a, b)?;

    let mut dot = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
    }
    Ok(-dot)
}

/// Dispatch vector distance calculation by the specified `Metric`.
pub fn distance(metric: Metric, a: &[f32], b: &[f32]) -> Result<f32, MetricError> {
    match metric {
        Metric::L2 => l2_distance(a, b),
        Metric::Cosine => cosine_distance(a, b),
        Metric::InnerProduct => inner_product_distance(a, b),
    }
}
