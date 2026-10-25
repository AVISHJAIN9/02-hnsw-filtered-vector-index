//! Data transfer objects for the Search API.

use std::collections::BTreeMap;

use hnsw_common::schema::PayloadValue;
use serde::{Deserialize, Serialize};

/// Search request JSON body payload.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchRequestDto {
    pub vector: Vec<f32>,
    pub k: usize,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
    #[serde(default)]
    pub include_payload: bool,
}

/// Individual scored result item in the response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredPointDto {
    pub id: u64,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<BTreeMap<String, PayloadValue>>,
}

/// Search response JSON body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponseDto {
    pub results: Vec<ScoredPointDto>,
    pub took_ms: f64,
}
