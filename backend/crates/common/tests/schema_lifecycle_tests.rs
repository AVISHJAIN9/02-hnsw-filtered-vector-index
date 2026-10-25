use hnsw_common::config::Config;
use hnsw_common::error::SchemaError;
use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use serial_test::serial;
use std::collections::BTreeSet;

#[test]
fn test_types_compilation_and_instantiation() {
    let id = VectorId(101);
    let dim = Dimension::new(4, 4096).expect("valid dimension");
    let metric = Metric::L2;
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: dim,
        metric,
    };
    assert_eq!(id.0, 101);
    assert_eq!(dim.get(), 4);
    assert_eq!(schema.dimension.get(), 4);
}

#[test]
fn test_serde_roundtrip_all_types() {
    let id = VectorId(999);
    let serialized_id = serde_json::to_string(&id).unwrap();
    let deserialized_id: VectorId = serde_json::from_str(&serialized_id).unwrap();
    assert_eq!(id, deserialized_id);

    let dim = Dimension::new(128, 4096).unwrap();
    let ser_dim = serde_json::to_string(&dim).unwrap();
    let de_dim: Dimension = serde_json::from_str(&ser_dim).unwrap();
    assert_eq!(dim, de_dim);

    for metric in [Metric::L2, Metric::Cosine, Metric::InnerProduct] {
        let ser = serde_json::to_string(&metric).unwrap();
        let de: Metric = serde_json::from_str(&ser).unwrap();
        assert_eq!(metric, de);
    }

    let mut set = BTreeSet::new();
    set.insert("alpha".to_string());
    set.insert("beta".to_string());

    let payload_values = vec![
        PayloadValue::String("hello".into()),
        PayloadValue::Int64(42),
        PayloadValue::Float64(12.345),
        PayloadValue::Bool(true),
        PayloadValue::StringSet(set),
    ];
    for pv in payload_values {
        let ser = serde_json::to_string(&pv).unwrap();
        let de: PayloadValue = serde_json::from_str(&ser).unwrap();
        assert_eq!(pv, de);
    }

    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(4, 4096).unwrap(),
        metric: Metric::Cosine,
    };
    let ser_schema = serde_json::to_string(&schema).unwrap();
    let de_schema: CollectionSchema = serde_json::from_str(&ser_schema).unwrap();
    assert_eq!(schema, de_schema);
}

#[test]
fn test_payload_value_integer_vs_float_roundtrip() {
    let int_json = "1";
    let pv_int: PayloadValue = serde_json::from_str(int_json).unwrap();
    assert_eq!(pv_int, PayloadValue::Int64(1));
    assert_eq!(serde_json::to_string(&pv_int).unwrap(), "1");

    let float_json = "1.0";
    let pv_float: PayloadValue = serde_json::from_str(float_json).unwrap();
    assert_eq!(pv_float, PayloadValue::Float64(1.0));
    assert_eq!(serde_json::to_string(&pv_float).unwrap(), "1.0");

    let nan_json = "NaN";
    assert!(serde_json::from_str::<PayloadValue>(nan_json).is_err());
    let inf_json = "Infinity";
    assert!(serde_json::from_str::<PayloadValue>(inf_json).is_err());
}

#[test]
fn test_golden_json_file() {
    let golden_str = include_str!("golden/vector_record_v1.json");
    let record: VectorRecord = serde_json::from_str(golden_str).expect("parse golden json");

    assert_eq!(record.id, VectorId(42));
    assert_eq!(record.schema_version, SchemaVersion::V1);
    assert_eq!(record.vector, vec![0.1, 0.2, 0.3, 0.4]);
    assert_eq!(record.version, 1);
    assert!(!record.deleted);

    let mut expected_tags = BTreeSet::new();
    expected_tags.insert("embedding".to_string());
    expected_tags.insert("vector".to_string());

    assert_eq!(
        record.payload.0.get("active"),
        Some(&PayloadValue::Bool(true))
    );
    assert_eq!(
        record.payload.0.get("count"),
        Some(&PayloadValue::Int64(100))
    );
    assert_eq!(
        record.payload.0.get("score"),
        Some(&PayloadValue::Float64(0.95))
    );
    assert_eq!(
        record.payload.0.get("tag"),
        Some(&PayloadValue::String("ml-test".into()))
    );
    assert_eq!(
        record.payload.0.get("tags"),
        Some(&PayloadValue::StringSet(expected_tags))
    );

    let re_serialized = serde_json::to_string_pretty(&record).unwrap();
    let re_parsed: VectorRecord = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(record, re_parsed);
}

#[test]
fn test_construct_rejects_dimension_zero() {
    let err = Dimension::new(0, 4096).unwrap_err();
    assert!(matches!(err, SchemaError::DimensionZero));
}

#[test]
fn test_construct_rejects_dimension_exceeding_max() {
    let err = Dimension::new(4097, 4096).unwrap_err();
    assert!(matches!(
        err,
        SchemaError::DimensionTooLarge {
            dim: 4097,
            max: 4096
        }
    ));
}

#[test]
fn test_construct_rejects_vector_len_mismatch() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(4, 4096).unwrap(),
        metric: Metric::L2,
    };
    let vector = vec![1.0, 2.0, 3.0]; // 3 items vs 4 expected
    let err = VectorRecord::new(&schema, VectorId(1), vector, Payload::default()).unwrap_err();
    assert!(matches!(
        err,
        SchemaError::DimensionMismatch {
            expected: 4,
            actual: 3
        }
    ));
}

#[test]
fn test_construct_rejects_non_finite_floats() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::L2,
    };

    let nan_vec = vec![1.0, f32::NAN, 3.0];
    let err_nan = VectorRecord::new(&schema, VectorId(1), nan_vec, Payload::default()).unwrap_err();
    assert!(matches!(err_nan, SchemaError::NonFiniteFloat));

    let inf_vec = vec![1.0, f32::INFINITY, 3.0];
    let err_inf = VectorRecord::new(&schema, VectorId(2), inf_vec, Payload::default()).unwrap_err();
    assert!(matches!(err_inf, SchemaError::NonFiniteFloat));

    let neg_inf_vec = vec![1.0, f32::NEG_INFINITY, 3.0];
    let err_neg_inf =
        VectorRecord::new(&schema, VectorId(3), neg_inf_vec, Payload::default()).unwrap_err();
    assert!(matches!(err_neg_inf, SchemaError::NonFiniteFloat));
}

#[test]
fn test_construct_rejects_zero_vector_for_cosine() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(3, 4096).unwrap(),
        metric: Metric::Cosine,
    };
    let zero_vec = vec![0.0, 0.0, 0.0];
    let err = VectorRecord::new(&schema, VectorId(1), zero_vec, Payload::default()).unwrap_err();
    assert!(matches!(err, SchemaError::ZeroVectorForCosine));
}

#[test]
fn test_deserialization_rejects_wrong_vector_length() {
    let bad_json = r#"{
        "schema_version": 1,
        "id": 10,
        "dimension": 4,
        "vector": [1.0, 2.0],
        "payload": {},
        "version": 1,
        "deleted": false
    }"#;
    let err_mismatch = serde_json::from_str::<VectorRecord>(bad_json).unwrap_err();
    assert!(err_mismatch
        .to_string()
        .contains("vector dimension mismatch"));

    let nan_json = r#"{
        "schema_version": 1,
        "id": 10,
        "vector": [1.0, null],
        "payload": {},
        "version": 1,
        "deleted": false
    }"#;
    assert!(serde_json::from_str::<VectorRecord>(nan_json).is_err());
}

#[test]
fn test_lifecycle_new_record_initial_state() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    assert_eq!(record.version, 1);
    assert!(!record.deleted);
}

#[test]
fn test_lifecycle_update_increments_version() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let mut record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    assert_eq!(record.version, 1);

    let mut payload = Payload::default();
    payload
        .0
        .insert("k".into(), PayloadValue::String("v".into()));
    record
        .update(&schema, vec![3.0, 4.0], payload.clone())
        .unwrap();

    assert_eq!(record.version, 2);
    assert_eq!(record.vector, vec![3.0, 4.0]);
    assert_eq!(record.payload, payload);
    assert!(!record.deleted);
}

#[test]
fn test_lifecycle_tombstone_marks_deleted() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let mut record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    record.tombstone().unwrap();
    assert!(record.deleted);
    assert_eq!(record.version, 2);
}

#[test]
fn test_lifecycle_tombstone_already_deleted_fails() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let mut record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    record.tombstone().unwrap();
    assert_eq!(record.version, 2);

    let err = record.tombstone().unwrap_err();
    assert!(matches!(err, SchemaError::AlreadyDeleted(VectorId(1))));
    assert_eq!(record.version, 2); // Version must NOT bump on already deleted
}

#[test]
fn test_lifecycle_update_after_tombstone_clears_deleted() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let mut record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    record.tombstone().unwrap();
    assert!(record.deleted);
    assert_eq!(record.version, 2);

    record
        .update(&schema, vec![5.0, 6.0], Payload::default())
        .unwrap();
    assert!(!record.deleted);
    assert_eq!(record.version, 3);
    assert_eq!(record.vector, vec![5.0, 6.0]);
}

#[test]
fn test_older_schema_version_rejected() {
    let v0_json = r#"{
        "schema_version": 0,
        "id": 1,
        "vector": [1.0, 2.0],
        "payload": {},
        "version": 1,
        "deleted": false
    }"#;
    let err_v0 = serde_json::from_str::<VectorRecord>(v0_json).unwrap_err();
    assert!(
        err_v0.to_string().contains("unsupported schema version")
            || err_v0.to_string().contains("UnsupportedVersion")
    );

    let v2_json = r#"{
        "schema_version": 2,
        "id": 1,
        "vector": [1.0, 2.0],
        "payload": {},
        "version": 1,
        "deleted": false
    }"#;
    let err_v2 = serde_json::from_str::<VectorRecord>(v2_json).unwrap_err();
    assert!(
        err_v2.to_string().contains("unsupported schema version")
            || err_v2.to_string().contains("UnsupportedVersion")
    );
}

#[test]
#[serial]
fn test_config_env_vars_and_reasons() {
    let default_cfg = Config::default();
    assert_eq!(default_cfg.max_dimension, 4096);
    assert_eq!(default_cfg.max_k, 1000);
    assert_eq!(default_cfg.max_filter_depth, 10);
    assert_eq!(default_cfg.max_filter_clauses, 100);
    assert_eq!(default_cfg.max_body_bytes, 10485760);

    // Verify overrides with HNSW_ prefix
    std::env::set_var("HNSW_MAX_DIMENSION", "1536");
    std::env::set_var("HNSW_MAX_K", "500");
    let env_cfg = Config::from_env();
    assert_eq!(env_cfg.max_dimension, 1536);
    assert_eq!(env_cfg.max_k, 500);

    std::env::remove_var("HNSW_MAX_DIMENSION");
    std::env::remove_var("HNSW_MAX_K");
}

#[test]
fn test_version_counter_overflow_returns_error() {
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(2, 4096).unwrap(),
        metric: Metric::L2,
    };
    let mut record =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0], Payload::default()).unwrap();
    record.version = u64::MAX;

    let err_update = record
        .update(&schema, vec![3.0, 4.0], Payload::default())
        .unwrap_err();
    assert!(matches!(err_update, SchemaError::VersionOverflow));

    let err_tombstone = record.tombstone().unwrap_err();
    assert!(matches!(err_tombstone, SchemaError::VersionOverflow));
}

