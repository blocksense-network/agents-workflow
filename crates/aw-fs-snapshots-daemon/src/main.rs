use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, warn, error};

mod server;
mod types;
mod operations;

use server::{DaemonServer, decode_length_prefixed_json, encode_length_prefixed_json};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Unix socket for listening
    #[arg(long, default_value = "/tmp/agent-workflow/aw-fs-snapshots-daemon")]
    socket_path: PathBuf,

    /// Run in stdin mode (read SSZ commands from stdin instead of socket)
    #[arg(long)]
    stdin_mode: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let level = match args.log_level.as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    info!("Starting AW filesystem snapshots daemon");

    if args.stdin_mode {
        info!("Running in stdin mode");
        run_stdin_mode().await?;
    } else {
        info!("Running in socket mode, socket path: {}", args.socket_path.display());
        run_socket_mode(args.socket_path).await?;
    }

    Ok(())
}

async fn run_socket_mode(socket_path: PathBuf) -> Result<()> {
    let mut server = DaemonServer::new(socket_path)?;

    // Set up signal handlers for graceful shutdown
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                return Err(e);
            }
        }
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down...");
            server.shutdown().await?;
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down...");
            server.shutdown().await?;
        }
    }

    Ok(())
}

async fn run_stdin_mode() -> Result<()> {
    use tokio::io::{stdin, AsyncBufReadExt, BufReader};
    use types::{Request, Response};

    let stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse length-prefixed JSON request from hex string
        let request_bytes = hex::decode(&line)?;
        let request: Request = decode_length_prefixed_json(&request_bytes)?;

        // Process the request
        let response = operations::process_request(request).await;

        // Encode response as length-prefixed JSON and output as hex
        let response_bytes = encode_length_prefixed_json(&response)?;
        println!("{}", hex::encode(&response_bytes));
    }

    Ok(())
}
