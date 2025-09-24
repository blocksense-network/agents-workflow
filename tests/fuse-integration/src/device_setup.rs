//! Device setup utilities for FUSE integration tests

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "fuse-device-setup")]
#[command(about = "Device setup utilities for FUSE integration tests")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Create a test block device
    Create {
        /// Device size in MB
        #[arg(short, long, default_value = "100")]
        size_mb: u32,

        /// Device name
        #[arg(short, long, default_value = "test_device")]
        name: String,
    },
    /// Clean up test block device
    Cleanup {
        /// Device name
        #[arg(short, long, default_value = "test_device")]
        name: String,
    },
    /// List existing test devices
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    match args.command {
        Commands::Create { size_mb, name } => {
            create_test_device(size_mb, &name).await?;
        }
        Commands::Cleanup { name } => {
            cleanup_test_device(&name).await?;
        }
        Commands::List => {
            list_test_devices().await?;
        }
    }

    Ok(())
}

async fn create_test_device(size_mb: u32, name: &str) -> Result<()> {
    info!("Creating test device '{}' with size {}MB", name, size_mb);

    let device_path = PathBuf::from(format!("/tmp/{}.img", name));

    // Check if device already exists
    if device_path.exists() {
        warn!("Device {} already exists, removing first", device_path.display());
        fs::remove_file(&device_path)?;
    }

    // Create sparse file
    let size_bytes = size_mb as u64 * 1024 * 1024;
    Command::new("truncate")
        .args(&["-s", &format!("{}", size_bytes), &device_path.to_string_lossy()])
        .status()
        .context("Failed to create sparse file")?;

    // Format as ext4 (Linux) or create HFS+ (macOS)
    #[cfg(target_os = "linux")]
    {
        Command::new("mkfs.ext4")
            .args(&["-F", &device_path.to_string_lossy()])
            .status()
            .context("Failed to format device as ext4")?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("hdiutil")
            .args(&["create", "-size", &format!("{}m", size_mb), "-fs", "HFS+", "-volname", name, &format!("/tmp/{}.dmg", name)])
            .status()
            .context("Failed to create HFS+ disk image")?;
    }

    info!("✅ Test device '{}' created successfully", name);
    Ok(())
}

async fn cleanup_test_device(name: &str) -> Result<()> {
    info!("Cleaning up test device '{}'", name);

    let img_path = PathBuf::from(format!("/tmp/{}.img", name));
    let dmg_path = PathBuf::from(format!("/tmp/{}.dmg", name));

    // Remove image file
    if img_path.exists() {
        fs::remove_file(&img_path)?;
        info!("Removed {}", img_path.display());
    }

    // Remove dmg file (macOS)
    if dmg_path.exists() {
        fs::remove_file(&dmg_path)?;
        info!("Removed {}", dmg_path.display());
    }

    info!("✅ Test device '{}' cleaned up", name);
    Ok(())
}

async fn list_test_devices() -> Result<()> {
    info!("Listing test devices in /tmp/");

    let entries = fs::read_dir("/tmp/")?;
    let mut found_devices = false;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();

        if name.ends_with(".img") || name.ends_with(".dmg") {
            if let Ok(metadata) = entry.metadata() {
                let size_mb = metadata.len() / (1024 * 1024);
                println!("{} ({}MB)", path.display(), size_mb);
                found_devices = true;
            }
        }
    }

    if !found_devices {
        println!("No test devices found");
    }

    Ok(())
}
