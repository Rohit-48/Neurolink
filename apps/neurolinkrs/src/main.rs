use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::path::Path;
use std::sync::Arc;
use axum::Router;
use clap::Parser;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod transfer;
mod api;
mod hashing;

use transfer::TransferManager;

#[derive(Parser, Debug)]
#[command(name = "neurolinkrs", version = "2.0.0", about = "Rust file sharing server with built-in web UI")]
struct Args {
    /// Port to run the server on
    #[arg(short, long, env = "NEUROLINKRS_PORT", default_value_t = 3001)]
    port: u16,

    /// Directory to store and serve shared files
    #[arg(short, long, env = "NEUROLINKRS_STORAGE", default_value = "./shared")]
    storage: String,
}

fn detect_lan_ip() -> Option<IpAddr> {
    // UDP connect lets us inspect the preferred outbound interface IP.
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    (!local_addr.ip().is_loopback()).then_some(local_addr.ip())
}

fn is_unsafe_browser_port(port: u16) -> bool {
    const UNSAFE_PORTS: &[u16] = &[
        1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
        101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179,
        389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563, 587, 601,
        636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061, 6000, 6566,
        6665, 6666, 6667, 6668, 6669, 6697, 10080,
    ];
    UNSAFE_PORTS.contains(&port)
}

fn print_elephant_banner() {
    let cmd = std::env::args()
        .next()
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("neurolinkrs")
        .to_string();
    let label = if cmd.contains("neurolinkd") {
        "NEUROLINKD 2.0 - Rust Daemon"
    } else {
        "NEUROLINKRS 2.0 - Rust Service"
    };

    let art = [
        "                       __     __",
        "                     /`  `\\_/`  \\",
        "                    /  _  _   _  \\",
        "                    | (o)(o) (o) |",
        "                    |      ^     |",
        "                    |  '\\___/'   |",
        "                     \\___________/",
        "                        /  |  \\",
        "                       /___|___\\",
    ];
    let colors = [97, 37, 96, 37, 97, 37, 96, 37, 97];

    println!();
    for (line, color) in art.iter().zip(colors.iter()) {
        println!("\x1b[1;38;5;{}m{}\x1b[0m", color, line);
    }
    println!("\x1b[1;97m                     {}\x1b[0m", label);
    println!();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging with filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    print_elephant_banner();

    let port = if is_unsafe_browser_port(args.port) {
        warn!(
            "Port {} is blocked by browsers (unsafe port list). Falling back to 3001.",
            args.port
        );
        3001
    } else {
        args.port
    };
    let storage_path = args.storage;
    tokio::fs::create_dir_all(&storage_path)
        .await
        .expect("Failed to create storage directory");

    info!("Starting NeuroLinkRS Rust Service v2.0.0");
    info!("Storage path: {}", storage_path);
    info!("Listening on port: {}", port);

    // Initialize transfer manager
    let transfer_manager = Arc::new(TransferManager::new(&storage_path));

    // Build router
    let app = Router::new()
        .merge(api::routes::routes(transfer_manager))
        .nest_service("/shared", ServeDir::new(storage_path.clone()))
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Server bind address: http://{}", addr);
    info!("Local access URL: http://localhost:{}", port);
    match detect_lan_ip() {
        Some(ip) => info!("Mobile/LAN access URL: http://{}:{}", ip, port),
        None => warn!("Could not determine LAN IP. Use your machine IP like http://192.168.x.x:{} from mobile", port),
    }
    info!("Web UI: http://localhost:{}/", port);
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
