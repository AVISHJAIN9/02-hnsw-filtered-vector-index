//! Canonicalization engine for deterministic filter representations.

use crate::ast::{Clause, Filter, MatchPredicate, Predicate};

/// Produces a canonical, flattened, and deterministically sorted representation of a Filter AST.
pub fn canonicalize(filter: &Filter) -> Filter {
    let mut canonical_must = Vec::new();
    for clause in &filter.must {
        match clause {
            Clause::Nested(inner) if inner.should.is_empty() && inner.must_not.is_empty() => {
                let canonical_inner = canonicalize(inner);
                canonical_must.extend(canonical_inner.must);
            }
            Clause::Nested(inner) => {
                let canonical_inner = canonicalize(inner);
                canonical_must.push(Clause::Nested(canonical_inner));
            }
            Clause::Condition(cond) => {
                canonical_must.push(Clause::Condition(canonicalize_condition(cond)));
            }
        }
    }
    canonical_must.sort();

    let mut canonical_should = Vec::new();
    for clause in &filter.should {
        match clause {
            Clause::Nested(inner) => {
                let canonical_inner = canonicalize(inner);
                if canonical_inner.should.len() == 1
                    && canonical_inner.must.is_empty()
                    && canonical_inner.must_not.is_empty()
                {
                    canonical_should.push(canonical_inner.should.into_iter().next().unwrap());
                } else {
                    canonical_should.push(Clause::Nested(canonical_inner));
                }
            }
            Clause::Condition(cond) => {
                canonical_should.push(Clause::Condition(canonicalize_condition(cond)));
            }
        }
    }
    canonical_should.sort();

    let mut canonical_must_not = Vec::new();
    for clause in &filter.must_not {
        match clause {
            Clause::Nested(inner) => {
                canonical_must_not.push(Clause::Nested(canonicalize(inner)));
            }
            Clause::Condition(cond) => {
                canonical_must_not.push(Clause::Condition(canonicalize_condition(cond)));
            }
        }
    }
    canonical_must_not.sort();

    Filter {
        must: canonical_must,
        should: canonical_should,
        must_not: canonical_must_not,
    }
}

fn canonicalize_condition(cond: &crate::ast::Condition) -> crate::ast::Condition {
    let mut cond = cond.clone();
    if let Predicate::Match(MatchPredicate::Any(ref mut list)) = cond.predicate {
        list.sort();
    }
    cond
}
