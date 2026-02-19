use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use console::style;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use reqwest::Client;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "neuroshare")]
#[command(about = "Send files to NeuroLink servers")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send files to a device
    Send {
        /// Files to send
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Target host
        #[arg(short = 'H', long, default_value = "localhost")]
        host: String,

        /// Target port
        #[arg(short, long, default_value = "3030")]
        port: u16,

        /// Chunk size in KB
        #[arg(short, long, default_value = "1024")]
        chunk_size: usize,
    },

    /// List available devices
    Devices {
        /// Discovery timeout in seconds
        #[arg(short, long, default_value = "5")]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { paths, host, port, chunk_size } => {
            send_files(paths, host, port, chunk_size).await?;
        }
        Commands::Devices { timeout } => {
            list_devices(timeout).await?;
        }
    }

    Ok(())
}

async fn send_files(paths: Vec<PathBuf>, host: String, port: u16, chunk_size_kb: usize) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let base_url = format!("http://{}:{}", host, port);

    println!("{}", style("NeuroShare").bold().cyan());
    println!("{}\n", style(format!("Sending to: {}:{}", host, port)).dim());

    for path in paths {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let metadata = tokio::fs::metadata(&path).await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let file_size = metadata.len();
        let chunk_size = chunk_size_kb * 1024;
        
        println!("{} {}", style("Sending:").bold(), style(filename).yellow());
        println!("  {} {}", style("Size:").dim(), format_size(file_size));

        // Initialize transfer
        let init_response: serde_json::Value = client
            .post(format!("{}/transfer/init", base_url))
            .json(&serde_json::json!({
                "filename": filename,
                "total_size": file_size,
                "chunk_size": chunk_size
            }))
            .send()
            .await?
            .json()
            .await?;

        if !init_response["success"].as_bool().unwrap_or(false) {
            println!("  {} {}", style("Error:").red().bold(), 
                init_response["error"].as_str().unwrap_or("Unknown error"));
            continue;
        }

        let transfer_id = init_response["data"]["transfer_id"].as_str().unwrap();
        let total_chunks = init_response["data"]["total_chunks"].as_u64().unwrap() as usize;

        // Create progress bar
        let pb = ProgressBar::new(file_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
            .unwrap()
            .progress_chars("#>-"));

        // Read and send chunks
        let mut file = File::open(&path).await?;
        let mut buffer = vec![0u8; chunk_size];
        let mut chunk_index = 0;
        let mut uploaded = 0u64;

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];

            // Create multipart form
            let form = reqwest::multipart::Form::new()
                .text("transfer_id", transfer_id.to_string())
                .text("chunk_index", chunk_index.to_string())
                .part("chunk", reqwest::multipart::Part::bytes(chunk_data.to_vec()));

            let response: serde_json::Value = client
                .post(format!("{}/transfer/chunk", base_url))
                .multipart(form)
                .send()
                .await?
                .json()
                .await?;

            if !response["success"].as_bool().unwrap_or(false) {
                pb.println(format!("  {} Chunk {} failed", style("Error:").red(), chunk_index));
                continue;
            }

            uploaded += bytes_read as u64;
            chunk_index += 1;
            
            pb.set_position(uploaded);
            pb.set_message(format!("Chunk {}/{}", chunk_index, total_chunks));
        }

        pb.finish_with_message("Upload complete, finalizing...");

        // Complete transfer
        let complete_response: serde_json::Value = client
            .post(format!("{}/transfer/complete", base_url))
            .json(&serde_json::json!({
                "transfer_id": transfer_id
            }))
            .send()
            .await?
            .json()
            .await?;

        if complete_response["success"].as_bool().unwrap_or(false) {
            println!("  {}\n", style("Success!").green().bold());
        } else {
            println!("  {} {}\n", style("Failed:").red().bold(),
                complete_response["error"].as_str().unwrap_or("Unknown error"));
        }
    }

    println!("{}", style("All transfers complete!").green().bold());
    Ok(())
}

async fn list_devices(timeout: u64) -> Result<()> {
    println!("{}", style("Discovering devices...").bold());
    println!("{}\n", style(format!("Scanning for {} seconds...", timeout)).dim());

    // TODO: Implement mDNS discovery
    println!("{}", style("mDNS discovery not yet implemented").yellow());
    println!("Use direct IP: neuroshare send file.txt --host <ip> --port <port>");

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
