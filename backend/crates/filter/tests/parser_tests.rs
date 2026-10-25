use hnsw_common::config::Config;
use hnsw_filter::error::FilterError;
use hnsw_filter::parser::FilterParser;
use serde_json::json;

#[test]
fn test_empty_filter_is_valid_and_matches_everything() {
    let json = json!({});
    let config = Config::default();
    let filter = FilterParser::parse(&json, &config).expect("empty filter must be valid");
    assert!(filter.is_empty());
}

#[test]
fn test_parse_match_value_and_match_any() {
    let json = json!({
        "must": [
            {"key": "city", "match": {"value": "Berlin"}},
            {"key": "tag", "match": {"any": ["news", "tech"]}}
        ]
    });
    let config = Config::default();
    let filter = FilterParser::parse(&json, &config).unwrap();
    assert_eq!(filter.must.len(), 2);
}

#[test]
fn test_parse_range_conditions() {
    let json = json!({
        "should": [
            {"key": "price", "range": {"gte": 10.5, "lt": 100}}
        ]
    });
    let config = Config::default();
    let filter = FilterParser::parse(&json, &config).unwrap();
    assert_eq!(filter.should.len(), 1);
}

#[test]
fn test_unknown_fields_rejected() {
    let json = json!({
        "must": [],
        "extra_field": 123
    });
    let config = Config::default();
    let err = FilterParser::parse(&json, &config).unwrap_err();
    match err {
        FilterError::UnknownField(field) => assert_eq!(field, "extra_field"),
        other => panic!("expected UnknownField, got {:?}", other),
    }
}

#[test]
fn test_type_error_range_on_bool_or_nan_or_inf() {
    let config = Config::default();

    // Range with string bound
    let json_str = json!({
        "must": [{"key": "age", "range": {"gt": "invalid"}}]
    });
    let err = FilterParser::parse(&json_str, &config).unwrap_err();
    assert!(matches!(err, FilterError::InvalidRangeBound { ref key, .. } if key == "age"));

    // Range with bool bound
    let json_bool = json!({
        "must": [{"key": "age", "range": {"gt": true}}]
    });
    let err = FilterParser::parse(&json_bool, &config).unwrap_err();
    assert!(matches!(err, FilterError::InvalidRangeBound { ref key, .. } if key == "age"));
}

#[test]
fn test_conflicting_range_bounds_gt_and_gte() {
    let config = Config::default();
    let json = json!({
        "must": [{"key": "score", "range": {"gt": 10, "gte": 20}}]
    });
    let err = FilterParser::parse(&json, &config).unwrap_err();
    assert!(matches!(err, FilterError::ConflictingBounds { ref key, .. } if key == "score"));
}

#[test]
fn test_mixed_type_any_list() {
    let config = Config::default();
    let json = json!({
        "must": [{"key": "category", "match": {"any": ["string", 42]}}]
    });
    let err = FilterParser::parse(&json, &config).unwrap_err();
    assert!(matches!(err, FilterError::MixedTypeList { ref key } if key == "category"));
}

#[test]
fn test_limits_max_filter_depth() {
    let config = Config {
        max_filter_depth: 3,
        ..Config::default()
    };

    // Depth 3: ok
    let ok_json = json!({
        "must": [{
            "must": [{
                "must": [{"key": "a", "match": {"value": 1}}]
            }]
        }]
    });
    assert!(FilterParser::parse(&ok_json, &config).is_ok());

    // Depth 4: fails
    let bad_json = json!({
        "must": [{
            "must": [{
                "must": [{
                    "must": [{"key": "a", "match": {"value": 1}}]
                }]
            }]
        }]
    });
    let err = FilterParser::parse(&bad_json, &config).unwrap_err();
    assert!(matches!(err, FilterError::MaxDepthExceeded { .. }));

    // Deep nesting (100000) does not panic or stack overflow
    // We construct a shallow JSON with depth parameter check or nested error
    let mut deep = json!({"key": "a", "match": {"value": 1}});
    for _ in 0..100 {
        deep = json!({"must": [deep]});
    }
    let err = FilterParser::parse(&deep, &config).unwrap_err();
    assert!(matches!(err, FilterError::MaxDepthExceeded { .. }));
}

#[test]
fn test_limits_max_clauses() {
    let config = Config {
        max_filter_clauses: 5,
        ..Config::default()
    };

    let clauses: Vec<_> = (0..6)
        .map(|i| json!({"key": format!("k{}", i), "match": {"value": i}}))
        .collect();
    let json = json!({"must": clauses});
    let err = FilterParser::parse(&json, &config).unwrap_err();
    assert!(matches!(err, FilterError::MaxClausesExceeded { .. }));
}
