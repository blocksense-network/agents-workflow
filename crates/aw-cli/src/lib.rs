//! Agents Workflow CLI library

pub mod agent;
pub mod task;
pub mod transport;

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
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// AgentFS filesystem operations
    Fs {
        #[command(subcommand)]
        subcommand: agent::fs::AgentFsCommands,
    },
}
