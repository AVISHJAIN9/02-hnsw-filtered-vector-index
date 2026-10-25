use std::collections::{BTreeMap, BTreeSet};

use hnsw_common::config::Config;
use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use hnsw_filter::evaluator::evaluate_filter;
use hnsw_filter::parser::FilterParser;
use serde_json::json;

fn dummy_record(payload_map: BTreeMap<String, PayloadValue>) -> VectorRecord {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2], Payload(payload_map)).unwrap()
}

#[test]
fn test_truth_table_must_evaluator() {
    let mut payload = BTreeMap::new();
    payload.insert("a".into(), PayloadValue::Int64(10));
    payload.insert("b".into(), PayloadValue::Int64(20));
    let rec = dummy_record(payload);

    let config = Config::default();

    // Both match -> true
    let f1 = FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": 10}}, {"key": "b", "match": {"value": 20}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f1, &rec));

    // One fails -> false
    let f2 = FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": 10}}, {"key": "b", "match": {"value": 99}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f2, &rec));
}

#[test]
fn test_truth_table_should_evaluator() {
    let mut payload = BTreeMap::new();
    payload.insert("color".into(), PayloadValue::String("red".into()));
    let rec = dummy_record(payload);

    let config = Config::default();

    // One matches -> true
    let f1 = FilterParser::parse(
        &json!({"should": [{"key": "color", "match": {"value": "red"}}, {"key": "color", "match": {"value": "blue"}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f1, &rec));

    // Neither matches -> false
    let f2 = FilterParser::parse(
        &json!({"should": [{"key": "color", "match": {"value": "green"}}, {"key": "color", "match": {"value": "blue"}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f2, &rec));

    // Empty should matches -> true
    let f3 = FilterParser::parse(&json!({"should": []}), &config).unwrap();
    assert!(evaluate_filter(&f3, &rec));
}

#[test]
fn test_truth_table_must_not_evaluator() {
    let mut payload = BTreeMap::new();
    payload.insert("active".into(), PayloadValue::Bool(true));
    let rec = dummy_record(payload);

    let config = Config::default();

    // Match rejected -> false
    let f1 = FilterParser::parse(
        &json!({"must_not": [{"key": "active", "match": {"value": true}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f1, &rec));

    // Does not match condition -> true
    let f2 = FilterParser::parse(
        &json!({"must_not": [{"key": "active", "match": {"value": false}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f2, &rec));
}

#[test]
fn test_missing_key_rule() {
    let payload = BTreeMap::new(); // empty payload
    let rec = dummy_record(payload);
    let config = Config::default();

    // Missing key does NOT match match predicate
    let f_match = FilterParser::parse(
        &json!({"must": [{"key": "nonexistent", "match": {"value": "val"}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_match, &rec));

    // Missing key does NOT match range predicate
    let f_range = FilterParser::parse(
        &json!({"must": [{"key": "nonexistent", "range": {"gt": 0}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_range, &rec));

    // Missing key DOES match must_not because it didn't match
    let f_must_not = FilterParser::parse(
        &json!({"must_not": [{"key": "nonexistent", "match": {"value": "val"}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_must_not, &rec));
}

#[test]
fn test_numeric_comparison_int_vs_float() {
    let mut payload = BTreeMap::new();
    payload.insert("val_int".into(), PayloadValue::Int64(1));
    payload.insert("val_float".into(), PayloadValue::Float64(1.0));
    let rec = dummy_record(payload);
    let config = Config::default();

    // 1 in int matches range gte 1.0 and lte 1.0
    let f1 = FilterParser::parse(
        &json!({"must": [{"key": "val_int", "range": {"gte": 1.0, "lte": 1.0}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f1, &rec));

    // 1.0 in float matches range gte 1 and lte 1
    let f2 = FilterParser::parse(
        &json!({"must": [{"key": "val_float", "range": {"gte": 1, "lte": 1}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f2, &rec));
}

#[test]
fn test_string_set_payload_matching() {
    let mut set = BTreeSet::new();
    set.insert("rust".to_string());
    set.insert("search".to_string());
    let mut payload = BTreeMap::new();
    payload.insert("tags".into(), PayloadValue::StringSet(set));
    let rec = dummy_record(payload);
    let config = Config::default();

    // match value in set
    let f1 = FilterParser::parse(
        &json!({"must": [{"key": "tags", "match": {"value": "rust"}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f1, &rec));

    // match value NOT in set
    let f2 = FilterParser::parse(
        &json!({"must": [{"key": "tags", "match": {"value": "python"}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f2, &rec));

    // match any intersects set
    let f3 = FilterParser::parse(
        &json!({"must": [{"key": "tags", "match": {"any": ["c++", "rust"]}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f3, &rec));

    // match any disjoint from set
    let f4 = FilterParser::parse(
        &json!({"must": [{"key": "tags", "match": {"any": ["c++", "java"]}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f4, &rec));
}
