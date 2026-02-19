use std::net::{IpAddr, SocketAddr, UdpSocket};
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

fn detect_lan_ip() -> Option<IpAddr> {
    // UDP connect lets us inspect the preferred outbound interface IP.
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    (!local_addr.ip().is_loopback()).then_some(local_addr.ip())
}

fn print_elephant_banner() {
    let art = [
        "                          _.-- ,.--.",
        "                        .'   .'    /",
        "                        | @       |'..--------._",
        "                       /      \\._/              '.",
        "                      /  .-.-                     \\",
        "                     (  /    \\                     \\",
        "                      \\\\      '.                  | #",
        "                       \\\\       \\   -.           /",
        "                        :\\       |    )._____.'   \\",
        "                         \"       |   /  \\  |  \\    )",
        "                                 |   |./'  :__ \\.-'",
        "                                 '--'",
    ];
    let colors = [51, 45, 39, 33, 27, 63, 99, 135, 141, 147, 153, 159];

    println!();
    for (line, color) in art.iter().zip(colors.iter()) {
        println!("\x1b[1;38;5;{}m{}\x1b[0m", color, line);
    }
    println!("\x1b[1;38;5;226m                 NEUROLINKD - Painted Elephant Mode\x1b[0m");
    println!();
}

#[tokio::main]
async fn main() {
    // Initialize logging with filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    print_elephant_banner();

    info!("🚀 Starting NeuroLink Rust Microservice v2.0.0");
    info!("📡 Local network file sharing with chunked transfers");
    info!("");

    // Get configuration from environment or defaults
    let port = std::env::var("NEUROLINK_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000u16);

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

    info!("Server bind address: http://{}", addr);
    info!("Local access URL: http://localhost:{}", port);
    match detect_lan_ip() {
        Some(ip) => info!("Mobile/LAN access URL: http://{}:{}", ip, port),
        None => warn!("Could not determine LAN IP. Use your machine IP like http://192.168.x.x:{} from mobile", port),
    }
    info!("Note: 0.0.0.0 is a listen address, not a browser URL.");

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
