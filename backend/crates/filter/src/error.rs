//! Filter parsing and evaluation errors.

use thiserror::Error;

/// Specific error conditions encountered when parsing or validating filter DSL expressions.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum FilterError {
    #[error("unknown field '{0}' in filter")]
    UnknownField(String),

    #[error("type mismatch for key '{key}': expected {expected}, got {actual}")]
    TypeMismatch {
        key: String,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("mixed-type list in 'any' match for key '{key}'")]
    MixedTypeList { key: String },

    #[error("invalid range bound for key '{key}': {reason}")]
    InvalidRangeBound { key: String, reason: &'static str },

    #[error("conflicting range bounds for key '{key}': {reason}")]
    ConflictingBounds { key: String, reason: &'static str },

    #[error("max filter depth exceeded: current depth {depth} exceeds max {max}")]
    MaxDepthExceeded { depth: usize, max: usize },

    #[error("max filter clauses exceeded: total clauses {clauses} exceeds max {max}")]
    MaxClausesExceeded { clauses: usize, max: usize },

    #[error("invalid filter format: {0}")]
    InvalidFormat(String),
}
