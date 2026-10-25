//! Centralized configuration and limits for the HNSW index system.

/// Global system limits overridable via environment variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Maximum vector dimension (default 4096: covers standard embedding models like OpenAI 3072d with headroom).
    pub max_dimension: u32,
    /// Maximum k nearest neighbors (default 1000: bounds candidate heap allocation and prevents server OOM).
    pub max_k: usize,
    /// Maximum nested filter depth (default 10: prevents recursive stack overflow on adversarial nested AST filters).
    pub max_filter_depth: usize,
    /// Maximum total filter clauses (default 100: bounds filter evaluation CPU budget per query).
    pub max_filter_clauses: usize,
    /// Maximum HTTP request body size in bytes (default 10MB: prevents memory exhaustion from oversized HTTP payloads).
    pub max_body_bytes: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_dimension: 4096,
            max_k: 1000,
            max_filter_depth: 10,
            max_filter_clauses: 100,
            max_body_bytes: 10 * 1024 * 1024,
        }
    }
}

impl Config {
    /// Load configuration from environment variables with HNSW_ prefix, falling back to defaults.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("HNSW_MAX_DIMENSION") {
            if let Ok(val) = v.parse::<u32>() {
                cfg.max_dimension = val;
            }
        }
        if let Ok(v) = std::env::var("HNSW_MAX_K") {
            if let Ok(val) = v.parse::<usize>() {
                cfg.max_k = val;
            }
        }
        if let Ok(v) = std::env::var("HNSW_MAX_FILTER_DEPTH") {
            if let Ok(val) = v.parse::<usize>() {
                cfg.max_filter_depth = val;
            }
        }
        if let Ok(v) = std::env::var("HNSW_MAX_FILTER_CLAUSES") {
            if let Ok(val) = v.parse::<usize>() {
                cfg.max_filter_clauses = val;
            }
        }
        if let Ok(v) = std::env::var("HNSW_MAX_BODY_BYTES") {
            if let Ok(val) = v.parse::<usize>() {
                cfg.max_body_bytes = val;
            }
        }
        cfg
    }
}
