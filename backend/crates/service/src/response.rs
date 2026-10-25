//! Response builder and scoring representation.

use hnsw_common::schema::{Metric, Payload, VectorId, VectorRecord};

/// A scored search result point.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredPoint {
    pub id: VectorId,
    pub score: f32,
    pub payload: Option<Payload>,
}

/// The search response returned to clients.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<ScoredPoint>,
    pub took_ms: f64,
}

/// Builds and calibrates search responses from retrieved records.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Compute similarity scores, sort descending, truncate to k, and assemble response.
    pub fn build(
        &self,
        records: Vec<VectorRecord>,
        k: usize,
        include_payload: bool,
        metric: Metric,
        query: &[f32],
        took_ms: f64,
    ) -> SearchResponse {
        let scored_points: Vec<ScoredPoint> = records
            .into_iter()
            .map(|record| {
                let score = Self::compute_score(metric, query, &record.vector);
                let payload = if include_payload {
                    Some(record.payload)
                } else {
                    None
                };
                ScoredPoint {
                    id: record.id,
                    score,
                    payload,
                }
            })
            .collect();

        let results = crate::top_k::top_k(scored_points, k).unwrap_or_default();

        SearchResponse { results, took_ms }
    }

    fn compute_score(metric: Metric, a: &[f32], b: &[f32]) -> f32 {
        match metric {
            Metric::L2 => {
                let dist_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
                1.0 / (1.0 + dist_sq)
            }
            Metric::Cosine => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
                }
            }
            Metric::InnerProduct => a.iter().zip(b.iter()).map(|(x, y)| x * y).sum(),
        }
    }
}
