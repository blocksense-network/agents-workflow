//! Agents Workflow CLI library

pub mod agent;
pub mod sandbox;
pub mod task;
pub mod transport;
pub mod tui;

// Re-export CLI types for testing
pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aw")]
#[command(about = "Agents Workflow CLI")]
#[command(version, author, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Task management commands
    Task {
        #[command(subcommand)]
        subcommand: task::TaskCommands,
    },
    /// Agent-related commands
    Agent {
        #[command(subcommand)]
        subcommand: AgentCommands,
    },
    /// Launch the Terminal User Interface
    Tui(tui::TuiArgs),
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// AgentFS filesystem operations
    Fs {
        #[command(subcommand)]
        subcommand: agent::fs::AgentFsCommands,
    },
    /// Run a command in a local sandbox
    Sandbox(sandbox::SandboxRunArgs),
}
