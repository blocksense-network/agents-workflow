use anyhow::Result;
use aw_cli::{Cli, Commands, AgentCommands, Parser};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Task { subcommand } => subcommand.run().await,
        Commands::Agent { subcommand } => match subcommand {
            AgentCommands::Fs { subcommand: cmd } => cmd.run().await,
        },
    }
}
