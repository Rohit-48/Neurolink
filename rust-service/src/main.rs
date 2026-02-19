use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod transfer;
mod api;
mod hashing;

use transfer::TransferManager;

#[tokio::main]
async fn main() {
    // Initialize logging with filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("🚀 Starting NeuroLink Rust Microservice v2.0.0");
    info!("📡 Local network file sharing with chunked transfers");
    info!("");

    // Get configuration from environment or defaults
    let port = std::env::var("NEUROLINK_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030u16);

    let storage_path = std::env::var("NEUROLINK_STORAGE")
        .unwrap_or_else(|_| "./shared".to_string());

    // Ensure storage directory exists
    tokio::fs::create_dir_all(&storage_path)
        .await
        .expect("Failed to create storage directory");

    info!("Storage path: {}", storage_path);
    info!("Listening on port: {}", port);

    // Initialize transfer manager
    let transfer_manager = Arc::new(TransferManager::new(&storage_path));

    // Build router
    let app = Router::new()
        .merge(api::routes::routes(transfer_manager))
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Server starting on http://{}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Server shutdown complete");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down..."),
        _ = terminate => info!("Received SIGTERM, shutting down..."),
    }
}
