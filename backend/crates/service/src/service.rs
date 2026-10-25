//! Core search service orchestrating validation, retrieval, filtering, and response assembly.

use std::time::Instant;

use hnsw_common::config::Config;
use hnsw_common::schema::CollectionSchema;

use crate::error::ServiceError;
use crate::filter::FilterEvaluator;
use crate::response::{ResponseBuilder, SearchResponse};
use crate::retriever::Retriever;
use crate::validator::Validator;

use hnsw_filter::ast::Filter;

/// Client query request for nearest neighbor search.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    pub k: usize,
    pub filter: Option<Filter>,
    pub include_payload: bool,
}

/// Search service composing validator, candidate retriever, filter evaluator, and response builder.
pub struct SearchService<R: Retriever, F: FilterEvaluator> {
    schema: CollectionSchema,
    config: Config,
    validator: Validator,
    retriever: R,
    filter_evaluator: F,
    response_builder: ResponseBuilder,
}

impl<R: Retriever, F: FilterEvaluator> SearchService<R, F> {
    /// Construct a new SearchService composing its required boundaries.
    pub fn new(
        schema: CollectionSchema,
        config: Config,
        validator: Validator,
        retriever: R,
        filter_evaluator: F,
        response_builder: ResponseBuilder,
    ) -> Self {
        Self {
            schema,
            config,
            validator,
            retriever,
            filter_evaluator,
            response_builder,
        }
    }

    /// Execute the search pipeline: validate -> retrieve -> filter -> build response.
    pub fn search(&self, req: SearchRequest) -> Result<SearchResponse, ServiceError> {
        let start = Instant::now();

        // 1. Validate query and parameters
        self.validator.validate(&req)?;

        // 2. Retrieve candidate records
        let candidates = self.retriever.retrieve(&req.vector, req.k)?;

        // 3. Filter candidate records (excluding tombstones and non-matching records)
        let mut filtered = Vec::with_capacity(candidates.len());
        for record in candidates {
            let req_filter_matches = match &req.filter {
                Some(f) => f.matches(&record)?,
                None => true,
            };
            if !record.deleted && self.filter_evaluator.matches(&record)? && req_filter_matches {
                filtered.push(record);
            }
        }

        // 4. Assemble, score, and format final response
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let response = self.response_builder.build(
            filtered,
            req.k,
            req.include_payload,
            self.schema.metric,
            &req.vector,
            elapsed_ms,
        );

        Ok(response)
    }

    /// Reference to the configured collection schema.
    pub fn schema(&self) -> &CollectionSchema {
        &self.schema
    }

    /// Reference to the active configuration limits.
    pub fn config(&self) -> &Config {
        &self.config
    }
}
