//! Simple test to verify mount operations work within user namespaces.
//!
//! This test isolates the mount capability issue by creating minimal namespaces
//! and attempting a simple mount operation.

use nix::mount::MsFlags;
use nix::unistd::Pid;
use tracing::{error, info};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting simple mount test in user namespace");

    // Test 1: Can we create user namespaces?
    info!("Testing user namespace creation...");
    match create_user_namespace() {
        Ok(pid) => {
            info!("✅ User namespace creation successful, child PID: {}", pid);
            // Wait for child to complete
            let status = nix::sys::wait::waitpid(pid, None)?;
            info!("Child exited with status: {:?}", status);
        }
        Err(e) => {
            error!("❌ User namespace creation failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn create_user_namespace() -> anyhow::Result<Pid> {
    // Fork to create child process that will enter namespaces
    match unsafe { nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Parent { child }) => {
            info!("Parent: Created child process {}", child);
            Ok(child)
        }
        Ok(nix::unistd::ForkResult::Child) => {
            info!("Child: Starting namespace creation...");

            // Create user namespace
            match nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWUSER) {
                Ok(_) => info!("Child: User namespace created successfully"),
                Err(e) => {
                    error!("Child: Failed to create user namespace: {}", e);
                    std::process::exit(1);
                }
            }

            // Test simple mount operation
            test_simple_mount();

            info!("Child: Exiting");
            std::process::exit(0);
        }
        Err(e) => Err(anyhow::anyhow!("Fork failed: {}", e)),
    }
}

fn test_simple_mount() {
    info!("Testing simple mount operation within user namespace...");

    // Create mount point directory
    if let Err(e) = std::fs::create_dir_all("/tmp/test_mount") {
        error!("Failed to create mount point directory: {}", e);
        return;
    }

    // Try to mount a simple tmpfs
    let result = nix::mount::mount(
        Some("tmpfs"),
        "/tmp/test_mount",
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
        Some("size=1m"),
    );

    match result {
        Ok(_) => {
            info!("✅ Simple mount operation succeeded!");
            info!("   This means CAP_SYS_ADMIN IS available in user namespace");

            // Try to unmount it
            let _ = nix::mount::umount("/tmp/test_mount");

            // Clean up directory
            let _ = std::fs::remove_dir("/tmp/test_mount");
        }
        Err(e) => {
            error!("❌ Simple mount operation failed: {}", e);
            if e.to_string().contains("EPERM") {
                error!("   This confirms CAP_SYS_ADMIN is not available in user namespace");
            } else {
                error!("   Unexpected error - may be mount point or other issue");
            }

            // Clean up directory
            let _ = std::fs::remove_dir("/tmp/test_mount");
        }
    }
}
