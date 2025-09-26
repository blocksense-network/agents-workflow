//! TUI binary entry point

use aw_rest_client::AuthConfig;
use aw_tui::App;
use clap::Parser;
use std::process;

/// TUI command-line arguments
#[derive(Parser)]
#[command(name = "aw-tui")]
#[command(about = "Terminal User Interface for Agents-Workflow")]
struct Args {
    /// Remote server URL (optional, falls back to local mode)
    #[arg(long)]
    remote_server: Option<String>,

    /// API key for authentication
    #[arg(long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    // Create REST client if remote server is specified
    let rest_client = if let Some(server_url) = args.remote_server {
        let auth = if let Some(api_key) = args.api_key {
            AuthConfig::with_api_key(api_key)
        } else {
            AuthConfig::default()
        };

        match aw_rest_client::RestClient::from_url(&server_url, auth) {
            Ok(client) => Some(client),
            Err(e) => {
                eprintln!("Failed to create REST client: {}", e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    // Create and run the TUI application
    let mut app = match App::new(rest_client) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize TUI: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = app.run().await {
        eprintln!("TUI application error: {}", e);
        process::exit(1);
    }
}
