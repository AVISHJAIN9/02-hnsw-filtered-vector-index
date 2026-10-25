use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use hnsw_common::schema::{
    CollectionSchema, Dimension, Metric, Payload, PayloadValue, SchemaVersion, VectorId,
    VectorRecord,
};
use hnsw_storage::error::StorageError;
use hnsw_storage::in_memory::InMemoryVectorStore;
use hnsw_storage::store::VectorStore;

fn sample_schema(dim: u32) -> CollectionSchema {
    CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(dim, 4096).unwrap(),
        metric: Metric::L2,
    }
}

#[test]
fn test_vector_storage_basic_put_get_len_iter_memory() {
    let schema = sample_schema(3);
    let store = InMemoryVectorStore::new(schema.clone());

    assert_eq!(store.len(), 0);
    assert_eq!(store.get(VectorId(1)), None);
    assert!(store.memory_bytes() > 0);

    let mut payload = Payload::default();
    payload
        .0
        .insert("tag".into(), PayloadValue::String("v1".into()));

    let rec1 =
        VectorRecord::new(&schema, VectorId(1), vec![1.0, 2.0, 3.0], payload.clone()).unwrap();
    let rec2 = VectorRecord::new(
        &schema,
        VectorId(2),
        vec![4.0, 5.0, 6.0],
        Payload::default(),
    )
    .unwrap();

    store.put(rec1.clone()).unwrap();
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(VectorId(1)), Some(rec1));

    store.put(rec2.clone()).unwrap();
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(VectorId(2)), Some(rec2));

    // iter over live records
    let live_records = store.iter();
    assert_eq!(live_records.len(), 2);
    assert_eq!(live_records[0].id, VectorId(1));
    assert_eq!(live_records[1].id, VectorId(2));

    assert!(store.memory_bytes() >= 2 * 3 * std::mem::size_of::<f32>());
}

#[test]
fn test_vectors_roundtrip_precision_and_bit_exactness() {
    // b. Round-trip test: store and fetch vectors containing f32::MIN_POSITIVE, f32::MAX, denormals, -0.0 and 1e-38; bit-exact equality via f32::to_bits.
    let special_vector = vec![
        f32::MIN_POSITIVE, // 1.17549435e-38
        f32::MAX,          // 3.40282347e+38
        1e-42_f32,         // denormal / subnormal float
        -0.0_f32,          // negative zero
        1e-38_f32,         // small positive normal float
    ];

    let schema = sample_schema(5);
    let store = InMemoryVectorStore::new(schema.clone());

    let record = VectorRecord::new(
        &schema,
        VectorId(42),
        special_vector.clone(),
        Payload::default(),
    )
    .unwrap();

    store.put(record).unwrap();

    let fetched = store.get(VectorId(42)).expect("record must exist");
    assert_eq!(fetched.vector.len(), special_vector.len());

    for (i, (&actual, &expected)) in fetched.vector.iter().zip(special_vector.iter()).enumerate() {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "bit-exact mismatch at index {}: actual {:032b} vs expected {:032b}",
            i,
            actual.to_bits(),
            expected.to_bits()
        );
    }
}

#[test]
fn test_duplicate_id_rejected_with_typed_error() {
    // c. put of an id that already exists returns a typed error
    let schema = sample_schema(2);
    let store = InMemoryVectorStore::new(schema.clone());

    let rec1 =
        VectorRecord::new(&schema, VectorId(99), vec![1.0, 2.0], Payload::default()).unwrap();
    let rec2 =
        VectorRecord::new(&schema, VectorId(99), vec![3.0, 4.0], Payload::default()).unwrap();

    store.put(rec1).unwrap();
    let err = store.put(rec2).unwrap_err();

    assert_eq!(err, StorageError::DuplicateId(VectorId(99)));
}

#[test]
fn test_schema_validation_on_put() {
    let schema = sample_schema(3);
    let store = InMemoryVectorStore::new(schema.clone());

    // Construct a record with wrong dimension (e.g. bypassing normal VectorRecord constructor or mismatched)
    let bad_record = VectorRecord {
        schema_version: SchemaVersion::V1,
        id: VectorId(101),
        vector: vec![1.0, 2.0], // len 2 vs expected 3
        payload: Payload::default(),
        version: 1,
        deleted: false,
    };

    let err = store.put(bad_record).unwrap_err();
    assert_eq!(
        err,
        StorageError::DimensionMismatch {
            expected: 3,
            actual: 2
        }
    );
}

#[test]
fn test_concurrency_8_readers_1_writer_no_torn_reads() {
    // d. Concurrency test: 8 reader threads and 1 writer thread for 1 second, no panics and no torn reads (every fetched vector equals one that was written).
    let schema = sample_schema(4);
    let store = Arc::new(InMemoryVectorStore::new(schema.clone()));
    let running = Arc::new(AtomicBool::new(true));
    let read_ops = Arc::new(AtomicUsize::new(0));

    // Valid vectors that the writer writes. Every element is i as f32 repeated 4 times: [i, i, i, i]
    // A torn read would be observing mixed values e.g. [1, 2, 1, 2].
    let store_writer = Arc::clone(&store);
    let running_writer = Arc::clone(&running);
    let writer_handle = std::thread::spawn(move || {
        let mut id = 1u64;
        while running_writer.load(Ordering::Relaxed) {
            let val = id as f32;
            let vec = vec![val, val, val, val];
            let rec = VectorRecord::new(&schema, VectorId(id), vec, Payload::default()).unwrap();
            let _ = store_writer.put(rec);
            id += 1;
            std::thread::yield_now();
        }
    });

    let mut reader_handles = Vec::new();
    for _ in 0..8 {
        let store_reader = Arc::clone(&store);
        let running_reader = Arc::clone(&running);
        let read_ops_reader = Arc::clone(&read_ops);

        reader_handles.push(std::thread::spawn(move || {
            let mut id = 1u64;
            while running_reader.load(Ordering::Relaxed) {
                if let Some(record) = store_reader.get(VectorId(id)) {
                    // Check for torn reads: every element in record.vector must be identical!
                    let expected_val = record.vector[0];
                    for &elem in &record.vector {
                        assert_eq!(
                            elem.to_bits(),
                            expected_val.to_bits(),
                            "Torn read detected for record {:?}! Vector was {:?}",
                            record.id,
                            record.vector
                        );
                    }
                    read_ops_reader.fetch_add(1, Ordering::Relaxed);
                    id += 1;
                } else if id > 1 {
                    id = 1;
                }
            }
        }));
    }

    // Run for 1 second
    let start = Instant::now();
    std::thread::sleep(Duration::from_secs(1));
    running.store(false, Ordering::Relaxed);

    writer_handle.join().unwrap();
    for rh in reader_handles {
        rh.join().unwrap();
    }

    let total_reads = read_ops.load(Ordering::Relaxed);
    println!(
        "Concurrency test completed in {:?}: {} valid reads, 0 torn reads, store len = {}",
        start.elapsed(),
        total_reads,
        store.len()
    );
    assert!(total_reads > 0, "Readers must successfully execute reads");
    assert!(store.len() > 0, "Writer must successfully write vectors");
}
