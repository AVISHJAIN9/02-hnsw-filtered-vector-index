//! Query and parameter validator.

use hnsw_common::config::Config;
use hnsw_common::error::SchemaError;
use hnsw_common::schema::{CollectionSchema, Metric};

use crate::error::ServiceError;
use crate::service::SearchRequest;

/// Validates search query dimensions, float values, metric invariants, and bounds.
#[derive(Debug, Clone)]
pub struct Validator {
    schema: CollectionSchema,
    config: Config,
}

impl Validator {
    /// Create a new validator bound to a specific schema and configuration.
    pub fn new(schema: CollectionSchema, config: Config) -> Self {
        Self { schema, config }
    }

    /// Validate the given search request against schema dimensions and configuration bounds.
    pub fn validate(&self, req: &SearchRequest) -> Result<(), ServiceError> {
        if req.k == 0 || req.k > self.config.max_k {
            return Err(ServiceError::InvalidK {
                k: req.k,
                max_k: self.config.max_k,
            });
        }

        let expected_dim = self.schema.dimension.get() as usize;
        if req.vector.len() != expected_dim {
            return Err(ServiceError::Schema(SchemaError::DimensionMismatch {
                expected: expected_dim,
                actual: req.vector.len(),
            }));
        }

        for &val in &req.vector {
            if !val.is_finite() {
                return Err(ServiceError::Schema(SchemaError::NonFiniteFloat));
            }
        }

        if self.schema.metric == Metric::Cosine && req.vector.iter().all(|&x| x == 0.0) {
            return Err(ServiceError::Schema(SchemaError::ZeroVectorForCosine));
        }

        Ok(())
    }
}
