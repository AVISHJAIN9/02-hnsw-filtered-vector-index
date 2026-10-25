use hnsw_common::config::Config;
use hnsw_filter::canonical::canonicalize;
use hnsw_filter::parser::FilterParser;
use proptest::prelude::*;
use serde_json::json;

#[test]
fn test_canonical_form_flattens_and_sorts_deterministically() {
    let json_a = json!({
        "must": [
            {"key": "z", "match": {"value": 1}},
            {"must": [{"key": "a", "match": {"value": 2}}]}
        ]
    });

    let config = Config::default();
    let filter_a = FilterParser::parse(&json_a, &config).unwrap();
    let canonical_a = canonicalize(&filter_a);
    let bytes_a = serde_json::to_vec(&canonical_a).unwrap();

    // Repeated 1000 times produces byte-identical results
    for _ in 0..1000 {
        let rec = canonicalize(&filter_a);
        let bytes = serde_json::to_vec(&rec).unwrap();
        assert_eq!(bytes_a, bytes);
    }
}

#[test]
fn test_logically_equivalent_clause_orders_produce_identical_canonical_form() {
    let config = Config::default();
    let json_1 = json!({
        "must": [
            {"key": "b", "match": {"value": 2}},
            {"key": "a", "match": {"value": 1}}
        ]
    });
    let json_2 = json!({
        "must": [
            {"key": "a", "match": {"value": 1}},
            {"key": "b", "match": {"value": 2}}
        ]
    });

    let f1 = FilterParser::parse(&json_1, &config).unwrap();
    let f2 = FilterParser::parse(&json_2, &config).unwrap();

    let c1 = canonicalize(&f1);
    let c2 = canonicalize(&f2);

    assert_eq!(
        serde_json::to_string(&c1).unwrap(),
        serde_json::to_string(&c2).unwrap()
    );
}

// Proptest generators for AST
prop_compose! {
    fn arb_condition()(key in "[a-z]{1,5}", val in any::<i64>()) -> serde_json::Value {
        json!({"key": key, "match": {"value": val}})
    }
}

proptest! {
    #[test]
    fn test_proptest_parse_canonical_roundtrip(
        conds in prop::collection::vec(arb_condition(), 1..5)
    ) {
        let config = Config::default();
        let input = json!({ "must": conds });
        let parsed = FilterParser::parse(&input, &config).unwrap();
        let canonical = canonicalize(&parsed);
        let serialized = serde_json::to_value(&canonical).unwrap();
        let reparsed = FilterParser::parse(&serialized, &config).unwrap();
        let recanonical = canonicalize(&reparsed);
        prop_assert_eq!(canonical, recanonical);
    }
}
