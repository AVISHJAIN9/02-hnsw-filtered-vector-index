//! Server setup, routing, and graceful shutdown.

use std::sync::Arc;

use axum::routing::post;
use axum::Router;
use hnsw_common::config::Config;
use hnsw_service::filter::FilterEvaluator;
use hnsw_service::retriever::Retriever;
use hnsw_service::service::SearchService;

use crate::handlers::{method_not_allowed_handler, not_found_handler, search_handler};

/// Application shared state.
pub struct AppState<R: Retriever + 'static, F: FilterEvaluator + 'static> {
    pub service: Arc<SearchService<R, F>>,
    pub config: Config,
}

/// Builds the Axum application router with configured endpoints, state, and fallbacks.
pub fn build_router<R, F>(state: Arc<AppState<R, F>>) -> Router
where
    R: Retriever + Send + Sync + 'static,
    F: FilterEvaluator + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/v1/search",
            post(search_handler).fallback(method_not_allowed_handler),
        )
        .fallback(not_found_handler)
        .with_state(state)
}

/// Serve the Axum router on a TCP listener with graceful shutdown.
pub async fn serve<F>(
    listener: tokio::net::TcpListener,
    app: Router,
    shutdown_signal: F,
) -> std::io::Result<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
}
