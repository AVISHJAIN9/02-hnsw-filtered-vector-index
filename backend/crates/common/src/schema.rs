//! Data model and lifecycle state representation for vectors, payloads, and collection schemas.

use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::error::SchemaError;

/// Strongly-typed 64-bit vector identifier.
///
/// Note: String and UUID identifiers are intentionally unsupported in this core schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VectorId(pub u64);

/// Bounded vector dimensionality in `1..=max_dimension`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dimension(u32);

impl Dimension {
    /// Create and validate a new `Dimension` instance.
    pub fn new(dim: u32, max_dimension: u32) -> Result<Self, SchemaError> {
        if dim == 0 {
            return Err(SchemaError::DimensionZero);
        }
        if dim > max_dimension {
            return Err(SchemaError::DimensionTooLarge {
                dim,
                max: max_dimension,
            });
        }
        Ok(Self(dim))
    }

    /// Retrieve the underlying dimension value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

/// Supported vector distance and similarity metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Metric {
    L2,
    Cosine,
    InnerProduct,
}

/// Supported payload field types.
#[derive(Debug, Clone, PartialEq)]
pub enum PayloadValue {
    String(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
    StringSet(BTreeSet<String>),
}

impl Serialize for PayloadValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PayloadValue::String(s) => serializer.serialize_str(s),
            PayloadValue::Int64(i) => serializer.serialize_i64(*i),
            PayloadValue::Float64(f) => serializer.serialize_f64(*f),
            PayloadValue::Bool(b) => serializer.serialize_bool(*b),
            PayloadValue::StringSet(set) => set.serialize(serializer),
        }
    }
}

struct PayloadValueVisitor;

impl<'de> Visitor<'de> for PayloadValueVisitor {
    type Value = PayloadValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, integer, finite float, bool, or set of strings")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PayloadValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PayloadValue::Int64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v <= i64::MAX as u64 {
            Ok(PayloadValue::Int64(v as i64))
        } else {
            Err(de::Error::custom("integer exceeds i64 range"))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.is_finite() {
            Ok(PayloadValue::Float64(v))
        } else {
            Err(de::Error::custom(
                "non-finite float in payload is not allowed",
            ))
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PayloadValue::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PayloadValue::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut set = BTreeSet::new();
        while let Some(elem) = seq.next_element::<String>()? {
            set.insert(elem);
        }
        Ok(PayloadValue::StringSet(set))
    }
}

impl<'de> Deserialize<'de> for PayloadValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PayloadValueVisitor)
    }
}

/// Metadata payload associating key-value attributes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Payload(pub BTreeMap<String, PayloadValue>);

/// Explicit schema version identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum SchemaVersion {
    V1 = 1,
}

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(*self as u32)
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = u32::deserialize(deserializer)?;
        match version {
            1 => Ok(SchemaVersion::V1),
            other => Err(de::Error::custom(SchemaError::UnsupportedVersion(other))),
        }
    }
}

/// Collection metadata schema defining vector dimensionality and metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub schema_version: SchemaVersion,
    pub dimension: Dimension,
    pub metric: Metric,
}

/// Unvalidated raw representation of `VectorRecord` for safe deserialization.
#[derive(Deserialize)]
struct RawVectorRecord {
    schema_version: u32,
    id: VectorId,
    vector: Vec<f32>,
    #[serde(default)]
    dimension: Option<u32>,
    #[serde(default)]
    payload: Payload,
    version: u64,
    deleted: bool,
}

impl TryFrom<RawVectorRecord> for VectorRecord {
    type Error = SchemaError;

    fn try_from(raw: RawVectorRecord) -> Result<Self, Self::Error> {
        if raw.schema_version != 1 {
            return Err(SchemaError::UnsupportedVersion(raw.schema_version));
        }

        if let Some(expected_dim) = raw.dimension {
            if raw.vector.len() != expected_dim as usize {
                return Err(SchemaError::DimensionMismatch {
                    expected: expected_dim as usize,
                    actual: raw.vector.len(),
                });
            }
        }

        for x in &raw.vector {
            if !x.is_finite() {
                return Err(SchemaError::NonFiniteFloat);
            }
        }

        Ok(VectorRecord {
            schema_version: SchemaVersion::V1,
            id: raw.id,
            vector: raw.vector,
            payload: raw.payload,
            version: raw.version,
            deleted: raw.deleted,
        })
    }
}

/// Validated vector record managing lifecycle state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawVectorRecord")]
pub struct VectorRecord {
    pub schema_version: SchemaVersion,
    pub id: VectorId,
    pub vector: Vec<f32>,
    pub payload: Payload,
    pub version: u64,
    pub deleted: bool,
}

impl VectorRecord {
    /// Construct a new vector record validating dimension and metric requirements.
    pub fn new(
        schema: &CollectionSchema,
        id: VectorId,
        vector: Vec<f32>,
        payload: Payload,
    ) -> Result<Self, SchemaError> {
        Self::validate_vector(schema, &vector)?;

        Ok(Self {
            schema_version: schema.schema_version,
            id,
            vector,
            payload,
            version: 1,
            deleted: false,
        })
    }

    /// Update vector and payload, incrementing version and clearing tombstone.
    pub fn update(
        &mut self,
        schema: &CollectionSchema,
        vector: Vec<f32>,
        payload: Payload,
    ) -> Result<(), SchemaError> {
        Self::validate_vector(schema, &vector)?;

        let next_version = self
            .version
            .checked_add(1)
            .ok_or(SchemaError::VersionOverflow)?;
        self.vector = vector;
        self.payload = payload;
        self.version = next_version;
        self.deleted = false;
        Ok(())
    }

    /// Tombstone the record, marking it as deleted and incrementing version.
    ///
    /// Calling tombstone on an already-deleted record returns `Err(SchemaError::AlreadyDeleted)`
    /// without incrementing the version counter.
    pub fn tombstone(&mut self) -> Result<(), SchemaError> {
        if self.deleted {
            return Err(SchemaError::AlreadyDeleted(self.id));
        }

        let next_version = self
            .version
            .checked_add(1)
            .ok_or(SchemaError::VersionOverflow)?;
        self.deleted = true;
        self.version = next_version;
        Ok(())
    }

    fn validate_vector(schema: &CollectionSchema, vector: &[f32]) -> Result<(), SchemaError> {
        let expected = schema.dimension.get() as usize;
        if vector.len() != expected {
            return Err(SchemaError::DimensionMismatch {
                expected,
                actual: vector.len(),
            });
        }

        for &val in vector {
            if !val.is_finite() {
                return Err(SchemaError::NonFiniteFloat);
            }
        }

        if schema.metric == Metric::Cosine && vector.iter().all(|&x| x == 0.0) {
            return Err(SchemaError::ZeroVectorForCosine);
        }

        Ok(())
    }
}
