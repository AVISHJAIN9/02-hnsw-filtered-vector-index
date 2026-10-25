//! Filter evaluation over VectorRecord payloads.

use hnsw_common::schema::{PayloadValue, VectorRecord};

use crate::ast::{Clause, Filter, MatchPredicate, Predicate, RangePredicate};

/// Evaluates whether a VectorRecord satisfies a Filter AST expression.
pub fn evaluate_filter(filter: &Filter, record: &VectorRecord) -> bool {
    // 1. must clauses (all must match)
    for clause in &filter.must {
        if !evaluate_clause(clause, record) {
            return false;
        }
    }

    // 2. should clauses (at least one must match; empty matches)
    if !filter.should.is_empty() {
        let mut matched = false;
        for clause in &filter.should {
            if evaluate_clause(clause, record) {
                matched = true;
                break;
            }
        }
        if !matched {
            return false;
        }
    }

    // 3. must_not clauses (none must match)
    for clause in &filter.must_not {
        if evaluate_clause(clause, record) {
            return false;
        }
    }

    true
}

fn evaluate_clause(clause: &Clause, record: &VectorRecord) -> bool {
    match clause {
        Clause::Nested(inner) => evaluate_filter(inner, record),
        Clause::Condition(cond) => {
            // Missing key never matches match or range predicate
            let payload_val = match record.payload.0.get(&cond.key) {
                Some(v) => v,
                None => return false,
            };

            match &cond.predicate {
                Predicate::Match(m) => evaluate_match(m, payload_val),
                Predicate::Range(r) => evaluate_range(r, payload_val),
            }
        }
    }
}

fn evaluate_match(predicate: &MatchPredicate, actual: &PayloadValue) -> bool {
    match predicate {
        MatchPredicate::Value(target) => values_match(actual, &target.0),
        MatchPredicate::Any(targets) => {
            targets.iter().any(|target| values_match(actual, &target.0))
        }
    }
}

fn values_match(actual: &PayloadValue, target: &PayloadValue) -> bool {
    match (actual, target) {
        (PayloadValue::StringSet(set), PayloadValue::String(s)) => set.contains(s),
        (PayloadValue::String(a), PayloadValue::String(b)) => a == b,
        (PayloadValue::Bool(a), PayloadValue::Bool(b)) => a == b,
        (PayloadValue::Int64(a), PayloadValue::Int64(b)) => a == b,
        (PayloadValue::Float64(a), PayloadValue::Float64(b)) => a == b,
        (PayloadValue::Int64(a), PayloadValue::Float64(b)) => (*a as f64) == *b,
        (PayloadValue::Float64(a), PayloadValue::Int64(b)) => *a == (*b as f64),
        _ => false,
    }
}

fn evaluate_range(range: &RangePredicate, actual: &PayloadValue) -> bool {
    let num_val = match actual {
        PayloadValue::Int64(i) => *i as f64,
        PayloadValue::Float64(f) => *f,
        _ => return false,
    };

    if let Some(gt) = range.gt {
        if num_val <= gt.0 {
            return false;
        }
    }
    if let Some(gte) = range.gte {
        if num_val < gte.0 {
            return false;
        }
    }
    if let Some(lt) = range.lt {
        if num_val >= lt.0 {
            return false;
        }
    }
    if let Some(lte) = range.lte {
        if num_val > lte.0 {
            return false;
        }
    }

    true
}
