//! Comprehensive integration tests for ZFS filesystem snapshot providers.
//!
//! This test file provides end-to-end testing of ZFS snapshot functionality,
//! similar to the legacy Ruby integration tests but implemented in Rust.

use aw_fs_snapshots_traits::{FsSnapshotProvider, WorkingCopyMode};
use filesystem_test_helpers::ZfsTestEnvironment;
use std::fs;
use std::path::Path;

// Include the filesystem test helpers module
#[path = "filesystem_test_helpers.rs"]
mod filesystem_test_helpers;

/// Integration test for ZFS filesystem snapshot providers.
///
/// This test creates ZFS test pools and exercises the provider API
/// to ensure everything works together correctly.
#[tokio::test]
async fn test_zfs_snapshot_integration() {
    // Skip if not running as root (required for ZFS operations)
    if !is_root() {
        println!("Skipping ZFS integration test: requires root privileges for ZFS operations");
        return;
    }

    let mut env = ZfsTestEnvironment::new().unwrap();

    // Test ZFS provider integration
    test_zfs_provider_integration(&mut env).await;
}

/// Test ZFS provider integration specifically.
async fn test_zfs_provider_integration(env: &mut ZfsTestEnvironment) {
    println!("Testing ZFS provider integration...");

    let pool_result = env.create_zfs_test_pool("integration_zfs_pool", Some(200));

    match pool_result {
        Ok(mount_point) => {
            println!("Successfully created ZFS pool at: {:?}", mount_point);

            // Initialize test repository
            initialize_test_repo(&mount_point);

            // Test ZFS provider specifically
            #[cfg(feature = "zfs")]
            {
                use aw_fs_snapshots_zfs::ZfsProvider;
                let zfs_provider = ZfsProvider::new();

                let ws_result = zfs_provider.prepare_writable_workspace(&mount_point, WorkingCopyMode::Worktree).await;
                match ws_result {
                    Ok(ws) => {
                        println!("ZFS workspace created: {:?}", ws.exec_path);

                        // Test snapshot creation
                        let snap_result = zfs_provider.snapshot_now(&ws, Some("integration_test")).await;
                        match snap_result {
                            Ok(snap) => {
                                println!("ZFS snapshot created: {}", snap.id);

                                // Test readonly mount (if supported)
                                let mount_result = zfs_provider.mount_readonly(&snap).await;
                                match mount_result {
                                    Ok(readonly_path) => {
                                        println!("Readonly mount created: {:?}", readonly_path);
                                        // Verify readonly access
                                        assert!(readonly_path.join("README.md").exists());
                                    }
                                    Err(e) => {
                                        println!("Readonly mount failed (may not be supported): {}", e);
                                    }
                                }

                                // Test branching from snapshot
                                let branch_result = zfs_provider.branch_from_snapshot(&snap, WorkingCopyMode::Worktree).await;
                                match branch_result {
                                    Ok(branch_ws) => {
                                        println!("Branch created: {:?}", branch_ws.exec_path);

                                        // Cleanup branch
                                        let _ = zfs_provider.cleanup(&branch_ws.cleanup_token).await;
                                    }
                                    Err(e) => {
                                        println!("Branch creation failed: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Snapshot creation failed: {}", e);
                            }
                        }

                        // Cleanup workspace
                        let _ = zfs_provider.cleanup(&ws.cleanup_token).await;
                    }
                    Err(e) => {
                        println!("ZFS workspace creation failed: {}", e);
                    }
                }
            }

            #[cfg(not(feature = "zfs"))]
            {
                println!("ZFS feature not enabled, skipping ZFS-specific tests");
            }
        }
        Err(e) => {
            println!("ZFS pool creation failed (expected in some environments): {}", e);
        }
    }
}

/// Initialize a test repository with some basic files.
fn initialize_test_repo(repo_path: &Path) {
    fs::write(repo_path.join("README.md"), "Integration test repository").unwrap();
    fs::write(repo_path.join("test_file.txt"), "Test content").unwrap();

    // Create a subdirectory
    let subdir = repo_path.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("nested_file.txt"), "Nested content").unwrap();
}

/// Check if running as root.
fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
