//! Parser for Filter DSL JSON expressions.

use hnsw_common::config::Config;
use hnsw_common::schema::PayloadValue;
use serde_json::Value;

use crate::ast::{
    Clause, Condition, Filter, MatchPredicate, OrderedFloat, Predicate, RangePredicate,
};
use crate::error::FilterError;

/// Parses JSON into validated, strongly-typed Filter AST.
pub struct FilterParser;

impl FilterParser {
    /// Parse JSON Value into Filter AST respecting configured limits and type constraints.
    pub fn parse(value: &Value, config: &Config) -> Result<Filter, FilterError> {
        let mut clause_count = 0;
        Self::parse_recursive(value, config, 1, &mut clause_count)
    }

    fn parse_recursive(
        value: &Value,
        config: &Config,
        depth: usize,
        clause_count: &mut usize,
    ) -> Result<Filter, FilterError> {
        if depth > config.max_filter_depth {
            return Err(FilterError::MaxDepthExceeded {
                depth,
                max: config.max_filter_depth,
            });
        }

        let obj = value.as_object().ok_or_else(|| {
            FilterError::InvalidFormat("filter must be a JSON object".to_string())
        })?;

        for key in obj.keys() {
            if key != "must" && key != "should" && key != "must_not" {
                return Err(FilterError::UnknownField(key.clone()));
            }
        }

        let mut filter = Filter::default();

        if let Some(must_val) = obj.get("must") {
            filter.must = Self::parse_clause_list(must_val, config, depth, clause_count)?;
        }
        if let Some(should_val) = obj.get("should") {
            filter.should = Self::parse_clause_list(should_val, config, depth, clause_count)?;
        }
        if let Some(must_not_val) = obj.get("must_not") {
            filter.must_not = Self::parse_clause_list(must_not_val, config, depth, clause_count)?;
        }

        Ok(filter)
    }

    fn parse_clause_list(
        value: &Value,
        config: &Config,
        depth: usize,
        clause_count: &mut usize,
    ) -> Result<Vec<Clause>, FilterError> {
        let list = value.as_array().ok_or_else(|| {
            FilterError::InvalidFormat("clause group must be an array".to_string())
        })?;

        let mut clauses = Vec::with_capacity(list.len());
        for item in list {
            *clause_count += 1;
            if *clause_count > config.max_filter_clauses {
                return Err(FilterError::MaxClausesExceeded {
                    clauses: *clause_count,
                    max: config.max_filter_clauses,
                });
            }

            let clause = Self::parse_clause(item, config, depth + 1, clause_count)?;
            clauses.push(clause);
        }

        Ok(clauses)
    }

    fn parse_clause(
        value: &Value,
        config: &Config,
        depth: usize,
        clause_count: &mut usize,
    ) -> Result<Clause, FilterError> {
        let obj = value.as_object().ok_or_else(|| {
            FilterError::InvalidFormat("clause must be a JSON object".to_string())
        })?;

        // If it contains a "key" field, it's a leaf Condition
        if let Some(key_val) = obj.get("key") {
            let key = key_val
                .as_str()
                .ok_or_else(|| FilterError::InvalidFormat("'key' must be a string".to_string()))?
                .to_string();

            // Validate known fields in Condition object
            for k in obj.keys() {
                if k != "key" && k != "match" && k != "range" {
                    return Err(FilterError::UnknownField(k.clone()));
                }
            }

            let predicate = match (obj.get("match"), obj.get("range")) {
                (Some(m), None) => Predicate::Match(Self::parse_match(m, &key)?),
                (None, Some(r)) => Predicate::Range(Self::parse_range(r, &key)?),
                (Some(_), Some(_)) => {
                    return Err(FilterError::InvalidFormat(
                        "cannot specify both 'match' and 'range' in a single condition".to_string(),
                    ));
                }
                (None, None) => {
                    return Err(FilterError::InvalidFormat(
                        "condition must specify either 'match' or 'range'".to_string(),
                    ));
                }
            };

            Ok(Clause::Condition(Condition { key, predicate }))
        } else {
            // Nested filter group
            let nested_filter = Self::parse_recursive(value, config, depth, clause_count)?;
            Ok(Clause::Nested(nested_filter))
        }
    }

    fn parse_match(value: &Value, key: &str) -> Result<MatchPredicate, FilterError> {
        let obj = value.as_object().ok_or_else(|| {
            FilterError::InvalidFormat("'match' must be a JSON object".to_string())
        })?;

        for k in obj.keys() {
            if k != "value" && k != "any" {
                return Err(FilterError::UnknownField(k.clone()));
            }
        }

        match (obj.get("value"), obj.get("any")) {
            (Some(v), None) => {
                let pv = Self::json_to_payload_value(v, key)?;
                Ok(MatchPredicate::Value(pv.into()))
            }
            (None, Some(a)) => {
                let arr = a.as_array().ok_or_else(|| {
                    FilterError::InvalidFormat("'match.any' must be an array".to_string())
                })?;
                let mut list = Vec::with_capacity(arr.len());
                let mut expected_discrim: Option<std::mem::Discriminant<PayloadValue>> = None;

                for item in arr {
                    let pv = Self::json_to_payload_value(item, key)?;
                    let disc = std::mem::discriminant(&pv);
                    if let Some(expected) = expected_discrim {
                        if disc != expected {
                            return Err(FilterError::MixedTypeList {
                                key: key.to_string(),
                            });
                        }
                    } else {
                        expected_discrim = Some(disc);
                    }
                    list.push(pv.into());
                }

                Ok(MatchPredicate::Any(list))
            }
            (Some(_), Some(_)) => Err(FilterError::InvalidFormat(
                "cannot specify both 'value' and 'any' in match".to_string(),
            )),
            (None, None) => Err(FilterError::InvalidFormat(
                "match must specify either 'value' or 'any'".to_string(),
            )),
        }
    }

    fn parse_range(value: &Value, key: &str) -> Result<RangePredicate, FilterError> {
        let obj = value.as_object().ok_or_else(|| {
            FilterError::InvalidFormat("'range' must be a JSON object".to_string())
        })?;

        for k in obj.keys() {
            if k != "gt" && k != "gte" && k != "lt" && k != "lte" {
                return Err(FilterError::UnknownField(k.clone()));
            }
        }

        if obj.contains_key("gt") && obj.contains_key("gte") {
            return Err(FilterError::ConflictingBounds {
                key: key.to_string(),
                reason: "cannot specify both 'gt' and 'gte'",
            });
        }
        if obj.contains_key("lt") && obj.contains_key("lte") {
            return Err(FilterError::ConflictingBounds {
                key: key.to_string(),
                reason: "cannot specify both 'lt' and 'lte'",
            });
        }

        let gt = obj
            .get("gt")
            .map(|v| Self::parse_number(v, key))
            .transpose()?;
        let gte = obj
            .get("gte")
            .map(|v| Self::parse_number(v, key))
            .transpose()?;
        let lt = obj
            .get("lt")
            .map(|v| Self::parse_number(v, key))
            .transpose()?;
        let lte = obj
            .get("lte")
            .map(|v| Self::parse_number(v, key))
            .transpose()?;

        Ok(RangePredicate {
            gt: gt.map(OrderedFloat),
            gte: gte.map(OrderedFloat),
            lt: lt.map(OrderedFloat),
            lte: lte.map(OrderedFloat),
        })
    }

    fn parse_number(value: &Value, key: &str) -> Result<f64, FilterError> {
        match value {
            Value::Number(num) => Ok(num.as_f64().unwrap_or(0.0)),
            Value::String(_) => Err(FilterError::InvalidRangeBound {
                key: key.to_string(),
                reason: "range bound cannot be a string",
            }),
            Value::Bool(_) => Err(FilterError::InvalidRangeBound {
                key: key.to_string(),
                reason: "range bound cannot be a boolean",
            }),
            _ => Err(FilterError::InvalidRangeBound {
                key: key.to_string(),
                reason: "range bound must be a number",
            }),
        }
    }

    fn json_to_payload_value(val: &Value, _key: &str) -> Result<PayloadValue, FilterError> {
        match val {
            Value::String(s) => Ok(PayloadValue::String(s.clone())),
            Value::Bool(b) => Ok(PayloadValue::Bool(*b)),
            Value::Number(num) => {
                if let Some(i) = num.as_i64() {
                    Ok(PayloadValue::Int64(i))
                } else {
                    Ok(PayloadValue::Float64(num.as_f64().unwrap_or(0.0)))
                }
            }
            _ => Err(FilterError::InvalidFormat(
                "payload value in match must be a string, number, or boolean".to_string(),
            )),
        }
    }
}
