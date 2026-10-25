use std::collections::{BTreeMap, BTreeSet};

use hnsw_common::config::Config;
use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use hnsw_filter::ast::{FilterValue, OrderedFloat};
use hnsw_filter::canonical::canonicalize;
use hnsw_filter::evaluator::evaluate_filter;
use hnsw_filter::parser::FilterParser;
use serde_json::json;

fn test_record(payload_map: BTreeMap<String, PayloadValue>) -> VectorRecord {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    VectorRecord::new(&schema, VectorId(1), vec![0.1, 0.2], Payload(payload_map)).unwrap()
}

#[test]
fn test_ast_helpers_coverage() {
    let pv_str = PayloadValue::String("hello".into());
    let fv_str: FilterValue = pv_str.clone().into();
    assert_eq!(&*fv_str, &pv_str);

    let fv_int: FilterValue = PayloadValue::Int64(42).into();
    let fv_float: FilterValue = PayloadValue::Float64(3.5).into();
    let fv_bool: FilterValue = PayloadValue::Bool(true).into();
    let mut set = BTreeSet::new();
    set.insert("a".into());
    let fv_set: FilterValue = PayloadValue::StringSet(set).into();

    // Test Ord branches
    assert!(fv_str < fv_int);
    assert!(fv_int < fv_float);
    assert!(fv_float < fv_bool);
    assert!(fv_bool < fv_set);

    let fv_str2: FilterValue = PayloadValue::String("world".into()).into();
    assert!(fv_str < fv_str2);

    let fv_int2: FilterValue = PayloadValue::Int64(100).into();
    assert!(fv_int < fv_int2);

    let fv_float2: FilterValue = PayloadValue::Float64(5.0).into();
    assert!(fv_float < fv_float2);

    let fv_bool2: FilterValue = PayloadValue::Bool(false).into();
    assert!(fv_bool2 < fv_bool);

    let mut set2 = BTreeSet::new();
    set2.insert("b".into());
    let fv_set2: FilterValue = PayloadValue::StringSet(set2).into();
    assert!(fv_set < fv_set2);

    // Test OrderedFloat helpers
    let of1 = OrderedFloat(1.5);
    let of2 = OrderedFloat(2.5);
    assert_eq!(of1, OrderedFloat(1.5));
    assert!(of1 < of2);
    assert_eq!(of1.cmp(&of2), std::cmp::Ordering::Less);
}

#[test]
fn test_canonical_nested_and_must_not_and_any_sorting() {
    let config = Config::default();

    // Nested should single clause
    let json_should = json!({
        "should": [
            {"should": [{"key": "a", "match": {"value": 1}}]},
            {"should": [{"key": "b", "match": {"value": 2}}, {"key": "c", "match": {"value": 3}}]}
        ]
    });
    let f_should = FilterParser::parse(&json_should, &config).unwrap();
    let c_should = canonicalize(&f_should);
    assert_eq!(c_should.should.len(), 2);

    // Nested must multiple clauses
    let json_must_multi = json!({
        "must": [
            {"should": [{"key": "x", "match": {"value": 1}}, {"key": "y", "match": {"value": 2}}]}
        ]
    });
    let f_must_multi = FilterParser::parse(&json_must_multi, &config).unwrap();
    let c_must_multi = canonicalize(&f_must_multi);
    assert_eq!(c_must_multi.must.len(), 1);

    // Nested must_not
    let json_must_not = json!({
        "must_not": [
            {"must": [{"key": "a", "match": {"value": 1}}]},
            {"key": "b", "match": {"any": [2, 1]}}
        ]
    });
    let f_must_not = FilterParser::parse(&json_must_not, &config).unwrap();
    let c_must_not = canonicalize(&f_must_not);
    assert_eq!(c_must_not.must_not.len(), 2);
}

#[test]
fn test_evaluator_all_branches() {
    let mut payload = BTreeMap::new();
    payload.insert("str".into(), PayloadValue::String("val".into()));
    payload.insert("bool".into(), PayloadValue::Bool(true));
    payload.insert("int".into(), PayloadValue::Int64(50));
    payload.insert("float".into(), PayloadValue::Float64(50.0));
    let rec = test_record(payload);
    let config = Config::default();

    // String match
    let f_str = FilterParser::parse(
        &json!({"must": [{"key": "str", "match": {"value": "val"}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_str, &rec));

    // Bool match
    let f_bool = FilterParser::parse(
        &json!({"must": [{"key": "bool", "match": {"value": true}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_bool, &rec));

    // Float with float match
    let f_flt = FilterParser::parse(
        &json!({"must": [{"key": "float", "match": {"value": 50.0}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_flt, &rec));

    // Int with float match
    let f_int_flt = FilterParser::parse(
        &json!({"must": [{"key": "int", "match": {"value": 50.0}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_int_flt, &rec));

    // Float with int match
    let f_flt_int = FilterParser::parse(
        &json!({"must": [{"key": "float", "match": {"value": 50}}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_flt_int, &rec));

    // String with int match (hits _ => false)
    let f_str_int = FilterParser::parse(
        &json!({"must": [{"key": "str", "match": {"value": 42}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_str_int, &rec));

    // Nested clause in evaluator
    let f_nested = FilterParser::parse(
        &json!({"must": [{"must": [{"key": "str", "match": {"value": "val"}}]}]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_nested, &rec));

    // Range on non-numeric payload (string or bool)
    let f_range_str = FilterParser::parse(
        &json!({"must": [{"key": "str", "range": {"gt": 10}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_range_str, &rec));

    // Range boundaries: gt, gte, lt, lte
    let f_range_bounds = FilterParser::parse(
        &json!({"must": [
            {"key": "int", "range": {"gt": 40, "lt": 60}},
            {"key": "float", "range": {"gte": 50.0, "lte": 50.0}}
        ]}),
        &config,
    )
    .unwrap();
    assert!(evaluate_filter(&f_range_bounds, &rec));

    // Range failing gt
    let f_fail_gt = FilterParser::parse(
        &json!({"must": [{"key": "int", "range": {"gt": 50}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_fail_gt, &rec));

    // Range failing lt
    let f_fail_lt = FilterParser::parse(
        &json!({"must": [{"key": "int", "range": {"lt": 50}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_fail_lt, &rec));

    // Range failing gte
    let f_fail_gte = FilterParser::parse(
        &json!({"must": [{"key": "int", "range": {"gte": 51}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_fail_gte, &rec));

    // Range failing lte
    let f_fail_lte = FilterParser::parse(
        &json!({"must": [{"key": "int", "range": {"lte": 49}}]}),
        &config,
    )
    .unwrap();
    assert!(!evaluate_filter(&f_fail_lte, &rec));
}

#[test]
fn test_parser_all_error_branches() {
    let config = Config::default();

    // 1. Not a JSON object at root
    assert!(FilterParser::parse(&json!([1, 2, 3]), &config).is_err());

    // 2. Clause group not an array
    assert!(FilterParser::parse(&json!({"must": "not_an_array"}), &config).is_err());

    // 3. Clause not an object
    assert!(FilterParser::parse(&json!({"must": ["not_an_obj"]}), &config).is_err());

    // 4. Key is not a string
    assert!(FilterParser::parse(&json!({"must": [{"key": 123}]}), &config).is_err());

    // 5. Unknown field in condition
    assert!(FilterParser::parse(&json!({"must": [{"key": "a", "unknown": 1}]}), &config).is_err());

    // 6. Both match and range in condition
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": 1}, "range": {"gt": 1}}]}),
        &config
    )
    .is_err());

    // 7. Neither match nor range in condition
    assert!(FilterParser::parse(&json!({"must": [{"key": "a"}]}), &config).is_err());

    // 8. Match is not an object
    assert!(
        FilterParser::parse(&json!({"must": [{"key": "a", "match": "str"}]}), &config).is_err()
    );

    // 9. Unknown field in match
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": 1, "bad": 2}}]}),
        &config
    )
    .is_err());

    // 10. Both value and any in match
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": 1, "any": [1]}}]}),
        &config
    )
    .is_err());

    // 11. Neither value nor any in match
    assert!(FilterParser::parse(&json!({"must": [{"key": "a", "match": {}}]}), &config).is_err());

    // 12. Match any is not an array
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"any": "not_array"}}]}),
        &config
    )
    .is_err());

    // 13. Range is not an object
    assert!(
        FilterParser::parse(&json!({"must": [{"key": "a", "range": "str"}]}), &config).is_err()
    );

    // 14. Unknown field in range
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "range": {"gt": 1, "bad": 2}}]}),
        &config
    )
    .is_err());

    // 15. Conflicting bounds lt and lte
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "range": {"lt": 1, "lte": 2}}]}),
        &config
    )
    .is_err());

    // 16. Range bound is object or array
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "range": {"gt": {}}}]}),
        &config
    )
    .is_err());

    // 17. Payload value in match is object or array
    assert!(FilterParser::parse(
        &json!({"must": [{"key": "a", "match": {"value": {}}}]}),
        &config
    )
    .is_err());
}
