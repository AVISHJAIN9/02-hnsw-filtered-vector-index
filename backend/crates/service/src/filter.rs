//! Filter evaluation boundary and real AST / no-filter implementations.

use hnsw_common::schema::VectorRecord;
use hnsw_filter::ast::Filter;

use crate::error::ServiceError;

/// Trait defining the filter evaluation contract over records.
pub trait FilterEvaluator: Send + Sync {
    /// Return true if the record satisfies the filter condition.
    fn matches(&self, record: &VectorRecord) -> Result<bool, ServiceError>;
}

/// Real no-op filter evaluator that accepts every record.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoFilter;

impl FilterEvaluator for NoFilter {
    fn matches(&self, _record: &VectorRecord) -> Result<bool, ServiceError> {
        Ok(true)
    }
}

/// Real AST-based filter evaluator evaluating against a Filter AST.
#[derive(Debug, Clone, Default)]
pub struct AstFilterEvaluator {
    pub filter: Filter,
}

impl AstFilterEvaluator {
    pub fn new(filter: Filter) -> Self {
        Self { filter }
    }
}

impl FilterEvaluator for AstFilterEvaluator {
    fn matches(&self, record: &VectorRecord) -> Result<bool, ServiceError> {
        Ok(hnsw_filter::evaluator::evaluate_filter(
            &self.filter,
            record,
        ))
    }
}

impl FilterEvaluator for Filter {
    fn matches(&self, record: &VectorRecord) -> Result<bool, ServiceError> {
        Ok(hnsw_filter::evaluator::evaluate_filter(self, record))
    }
}
