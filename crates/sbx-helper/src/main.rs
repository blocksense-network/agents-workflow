//! Sandbox helper binary that becomes PID 1 in the sandbox environment.

#![cfg(target_os = "linux")]

use clap::Parser;
use sandbox_core::{NamespaceConfig, ProcessConfig, Sandbox};
use sandbox_fs::{FilesystemConfig, FilesystemManager};
use tracing::{error, info};

/// Command line arguments for sbx-helper
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute in sandbox
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,

    /// Working directory for the command
    #[arg(short = 'C', long)]
    working_dir: Option<String>,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,

    /// Disable user namespace isolation
    ///
    /// Note: Disabling user namespaces may require CAP_SYS_ADMIN for other namespaces.
    /// User namespaces allow unprivileged creation of other namespace types.
    #[arg(long)]
    no_user_ns: bool,

    /// Disable mount namespace isolation
    ///
    /// Note: Mount operations require CAP_SYS_ADMIN (typically root privileges).
    #[arg(long)]
    no_mount_ns: bool,

    /// Disable PID namespace isolation
    ///
    /// Note: Creating PID namespaces requires CAP_SYS_ADMIN unless done within
    /// a user namespace created in the same unshare() call.
    #[arg(long)]
    no_pid_ns: bool,

    /// Read-write directory to allow in sandbox
    #[arg(long)]
    rw_dir: Option<String>,
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

    info!("Starting sandbox helper with args: {:?}", args);

    // Create sandbox configuration
    let namespace_config = NamespaceConfig {
        user_ns: !args.no_user_ns,
        mount_ns: !args.no_mount_ns,
        pid_ns: !args.no_pid_ns,
        uts_ns: true,
        ipc_ns: true,
        time_ns: false,
        uid_map: None,
        gid_map: None,
    };

    // Create process configuration
    let command = if args.command.is_empty() {
        vec!["/bin/sh".to_string()]
    } else {
        args.command.clone()
    };

    // Prepare environment variables for the sandboxed process
    let mut env_vars = std::env::vars().collect::<Vec<(String, String)>>();
    // Set environment variable to indicate we're running in a sandboxed test environment
    // This allows test programs to safely perform resource-intensive operations
    env_vars.push(("SANDBOX_TEST_MODE".to_string(), "1".to_string()));

    let process_config = ProcessConfig {
        command,
        working_dir: args.working_dir.clone(),
        env: env_vars,
    };

    // Create filesystem configuration
    let mut fs_config = FilesystemConfig::default();
    if let Some(rw_dir) = &args.rw_dir {
        fs_config.working_dir = Some(rw_dir.clone());
    }

    // Initialize sandbox with cgroups enabled
    let mut sandbox = Sandbox::with_namespace_config(namespace_config)
        .with_process_config(process_config)
        .with_default_cgroups();

    let fs_manager = FilesystemManager::with_config(fs_config);

    // Start sandbox (enter all namespaces in single unshare() call)
    // After this point, we are within the user namespace and have root privileges there
    sandbox.start()?;

    // Add current process to cgroup if cgroups are enabled
    sandbox.add_process_to_cgroup(std::process::id())?;

    // Set up filesystem isolation (executed within user namespace context)
    fs_manager.setup_mounts().await?;

    // Execute the process as PID 1
    // This will replace the current process
    match sandbox.exec_process() {
        Ok(_) => unreachable!("exec_process should not return"),
        Err(e) => {
            error!("Failed to execute process: {}", e);
            std::process::exit(1);
        }
    }
}
