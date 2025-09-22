//! AgentFS WinFsp Host — Windows filesystem adapter
//!
//! This binary implements a WinFsp host that mounts AgentFS volumes
//! on Windows using the WinFsp user-mode filesystem framework.

use anyhow::Result;
use std::path::PathBuf;

#[derive(clap::Parser)]
struct Args {
    /// Drive letter to mount (e.g., X:)
    mount_point: String,

    /// Configuration file (JSON)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    println!("AgentFS WinFsp Host - M1 bootstrap complete");
    println!("Mount point: {:?}", std::env::args().nth(1));
    Ok(())
}
