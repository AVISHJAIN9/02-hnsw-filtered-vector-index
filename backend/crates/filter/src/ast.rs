//! Typed Abstract Syntax Tree (AST) for the Filter DSL.

use std::ops::Deref;

use hnsw_common::schema::PayloadValue;
use serde::{Deserialize, Serialize};

/// Root filter structure composed of boolean clause groups.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Filter {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must: Vec<Clause>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub should: Vec<Clause>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must_not: Vec<Clause>,
}

impl Filter {
    /// Returns true if all clause groups are empty.
    pub fn is_empty(&self) -> bool {
        self.must.is_empty() && self.should.is_empty() && self.must_not.is_empty()
    }
}

/// A filter clause: either a leaf predicate condition or a nested filter group.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Clause {
    Condition(Condition),
    Nested(Filter),
}

/// A key-bound condition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    #[serde(flatten)]
    pub predicate: Predicate,
}

/// Condition predicate types.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Predicate {
    Match(MatchPredicate),
    Range(RangePredicate),
}

/// Wrapper around PayloadValue providing total Ord and Eq for canonical sorting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FilterValue(pub PayloadValue);

impl Deref for FilterValue {
    type Target = PayloadValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PayloadValue> for FilterValue {
    fn from(val: PayloadValue) -> Self {
        Self(val)
    }
}

impl Eq for FilterValue {}

impl PartialOrd for FilterValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FilterValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn tag(v: &PayloadValue) -> u8 {
            match v {
                PayloadValue::String(_) => 0,
                PayloadValue::Int64(_) => 1,
                PayloadValue::Float64(_) => 2,
                PayloadValue::Bool(_) => 3,
                PayloadValue::StringSet(_) => 4,
            }
        }
        match (&self.0, &other.0) {
            (PayloadValue::String(a), PayloadValue::String(b)) => a.cmp(b),
            (PayloadValue::Int64(a), PayloadValue::Int64(b)) => a.cmp(b),
            (PayloadValue::Float64(a), PayloadValue::Float64(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (PayloadValue::Bool(a), PayloadValue::Bool(b)) => a.cmp(b),
            (PayloadValue::StringSet(a), PayloadValue::StringSet(b)) => a.cmp(b),
            (a, b) => tag(a).cmp(&tag(b)),
        }
    }
}

/// Match predicate matching a single value or any value in a list.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchPredicate {
    Value(FilterValue),
    Any(Vec<FilterValue>),
}

/// Wrapped finite f64 for deterministic Ord and Eq.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrderedFloat(pub f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits() || self.0 == other.0
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Numeric range bounds.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RangePredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<OrderedFloat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<OrderedFloat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<OrderedFloat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<OrderedFloat>,
}
