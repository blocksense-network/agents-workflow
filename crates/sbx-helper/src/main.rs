//! Sandbox helper binary that becomes PID 1 in the sandbox environment.

#![cfg(target_os = "linux")]

use clap::Parser;
use tracing::info;

/// Command line arguments for sbx-helper
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    info!("Starting sandbox helper");

    // TODO: Initialize sandbox components
    // - Load configuration
    // - Set up namespaces
    // - Configure filesystem
    // - Install seccomp filters
    // - Set up cgroups
    // - Configure networking
    // - Execute entrypoint

    info!("Sandbox helper initialized successfully");

    Ok(())
}
