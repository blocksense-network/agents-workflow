//! AgentFS FUSE Integration Test Runner
//!
//! This binary provides comprehensive integration testing for the AgentFS FUSE adapter,
//! including mount/unmount cycles, filesystem operations, control plane functionality,
//! and pjdfstest compliance testing.

#[cfg(feature = "fuse")]
mod fuse_tests;
mod test_utils;

#[cfg(feature = "fuse")]
use fuse_tests::*;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "fuse-integration-tests")]
#[command(about = "AgentFS FUSE Integration Test Runner")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Test configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all integration tests
    All {
        /// Skip pjdfstest compliance tests
        #[arg(long)]
        skip_pjdfs: bool,

        /// Skip stress tests
        #[arg(long)]
        skip_stress: bool,
    },
    /// Test full mount cycle (create device → mount → operations → unmount → cleanup)
    MountCycle,
    /// Test filesystem operations through FUSE interface
    FsOps,
    /// Test control plane operations via .agentfs/control file
    ControlPlane,
    /// Run pjdfstest compliance tests
    Pjdfstest,
    /// Run stress and performance tests
    Stress,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let level = if args.verbose { "debug" } else { "info" };
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(format!("fuse_integration_tests={},agentfs=debug", level))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    info!(target: "fuse_integration_tests", "AgentFS FUSE Integration Test Runner starting");

    match args.command {
        Commands::All { skip_pjdfs, skip_stress } => {
            run_all_tests(skip_pjdfs, skip_stress).await?;
        }
        Commands::MountCycle => {
            run_mount_cycle_tests().await?;
        }
        Commands::FsOps => {
            run_filesystem_ops_tests().await?;
        }
        Commands::ControlPlane => {
            run_control_plane_tests().await?;
        }
        Commands::Pjdfstest => {
            run_pjdfstest_compliance().await?;
        }
        Commands::Stress => {
            run_stress_tests().await?;
        }
    }

    info!(target: "fuse_integration_tests", "All tests completed successfully");
    Ok(())
}

async fn run_all_tests(skip_pjdfs: bool, skip_stress: bool) -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        info!("Running all FUSE integration tests");

        // Test 1: Mount cycle
        run_mount_cycle_tests().await?;
        info!("✅ Mount cycle tests passed");

        // Test 2: Filesystem operations
        run_filesystem_ops_tests().await?;
        info!("✅ Filesystem operations tests passed");

        // Test 3: Control plane
        run_control_plane_tests().await?;
        info!("✅ Control plane tests passed");

        // Test 4: pjdfstest (optional)
        if !skip_pjdfs {
            run_pjdfstest_compliance().await?;
            info!("✅ pjdfstest compliance tests passed");
        } else {
            info!("⏭️  Skipping pjdfstest compliance tests");
        }

        // Test 5: Stress tests (optional)
        if !skip_stress {
            run_stress_tests().await?;
            info!("✅ Stress tests passed");
        } else {
            info!("⏭️  Skipping stress tests");
        }

        info!("🎉 All FUSE integration tests completed successfully!");
        Ok(())
    }

    #[cfg(not(feature = "fuse"))]
    {
        warn!(target: "fuse_integration_tests", "FUSE support not compiled in. Use --features fuse to enable FUSE testing.");
        info!(target: "fuse_integration_tests", "To run tests, use: cargo run -p fuse-integration-tests --features fuse --bin fuse_test_runner -- all");
        Ok(())
    }
}

async fn run_mount_cycle_tests() -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        fuse_tests::run_mount_cycle_tests().await
    }

    #[cfg(not(feature = "fuse"))]
    {
        info!("⏭️  Skipping mount cycle tests - FUSE support not compiled in");
        info!("To enable FUSE tests, compile with: cargo build --features fuse");
        Ok(())
    }
}

async fn run_filesystem_ops_tests() -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        fuse_tests::run_filesystem_ops_tests().await
    }

    #[cfg(not(feature = "fuse"))]
    {
        info!("⏭️  Skipping filesystem operations tests - FUSE support not compiled in");
        Ok(())
    }
}

async fn run_control_plane_tests() -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        fuse_tests::run_control_plane_tests().await
    }

    #[cfg(not(feature = "fuse"))]
    {
        info!("⏭️  Skipping control plane tests - FUSE support not compiled in");
        Ok(())
    }
}

async fn run_pjdfstest_compliance() -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        fuse_tests::run_pjdfstest_compliance().await
    }

    #[cfg(not(feature = "fuse"))]
    {
        info!("⏭️  Skipping pjdfstest compliance tests - FUSE support not compiled in");
        Ok(())
    }
}

async fn run_stress_tests() -> Result<()> {
    #[cfg(feature = "fuse")]
    {
        fuse_tests::run_stress_tests().await
    }

    #[cfg(not(feature = "fuse"))]
    {
        info!("⏭️  Skipping stress tests - FUSE support not compiled in");
        Ok(())
    }
}
