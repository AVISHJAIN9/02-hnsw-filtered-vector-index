//! HNSW Search API server executable.

use std::sync::Arc;

use hnsw_api::empty_retriever::EmptyIndexRetriever;
use hnsw_api::server::{build_router, serve, AppState};
use hnsw_common::config::Config;
use hnsw_common::schema::{CollectionSchema, Dimension, Metric, SchemaVersion};
use hnsw_service::filter::NoFilter;
use hnsw_service::response::ResponseBuilder;
use hnsw_service::service::SearchService;
use hnsw_service::validator::Validator;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => {
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            println!("Received SIGTERM, shutting down gracefully");
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let schema = CollectionSchema {
        schema_version: SchemaVersion::V1,
        dimension: Dimension::new(128, config.max_dimension)?,
        metric: Metric::Cosine,
    };
    let validator = Validator::new(schema.clone(), config.clone());
    let retriever = EmptyIndexRetriever;
    let service = SearchService::new(
        schema,
        config.clone(),
        validator,
        retriever,
        NoFilter,
        ResponseBuilder,
    );

    let state = Arc::new(AppState {
        service: Arc::new(service),
        config,
    });
    let app = build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server listening on http://{}", addr);

    serve(listener, app, shutdown_signal()).await?;
    println!("Server shutdown complete");

    Ok(())
}
