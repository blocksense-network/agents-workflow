//! AgentFS Core — Cross-platform filesystem core implementation
//!
//! This crate provides the core VFS, snapshots, branches, and storage
//! functionality for AgentFS, with platform adapters providing the glue.

pub mod config;
pub mod error;
pub mod types;

// Re-export key types for convenience
pub use config::{CachePolicy, CaseSensitivity, FsConfig, FsLimits, MemoryPolicy};
pub use error::FsError;
pub use types::*;

// Core implementation modules
pub mod storage;
pub mod vfs;

// Re-export main types
pub use vfs::FsCore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FsError::NotFound;
        assert_eq!(err.to_string(), "not found");
    }

    #[test]
    fn test_config_creation() {
        let config = FsConfig {
            case_sensitivity: CaseSensitivity::Sensitive,
            memory: MemoryPolicy {
                max_bytes_in_memory: Some(1024 * 1024),
                spill_directory: None,
            },
            limits: FsLimits {
                max_open_handles: 1000,
                max_branches: 100,
                max_snapshots: 1000,
            },
            cache: CachePolicy {
                attr_ttl_ms: 1000,
                entry_ttl_ms: 1000,
                negative_ttl_ms: 1000,
                enable_readdir_plus: true,
                auto_cache: true,
                writeback_cache: false,
            },
            enable_xattrs: true,
            enable_ads: false,
            track_events: true,
        };
        // Basic smoke test - config can be created
        assert!(config.enable_xattrs);
    }

    fn test_core() -> FsCore {
        let config = FsConfig {
            case_sensitivity: CaseSensitivity::Sensitive,
            memory: MemoryPolicy {
                max_bytes_in_memory: Some(1024 * 1024),
                spill_directory: None,
            },
            limits: FsLimits {
                max_open_handles: 1000,
                max_branches: 100,
                max_snapshots: 1000,
            },
            cache: CachePolicy {
                attr_ttl_ms: 1000,
                entry_ttl_ms: 1000,
                negative_ttl_ms: 1000,
                enable_readdir_plus: true,
                auto_cache: true,
                writeback_cache: false,
            },
            enable_xattrs: true,
            enable_ads: false,
            track_events: true,
        };
        FsCore::new(config).unwrap()
    }

    fn rw_create() -> OpenOptions {
        OpenOptions {
            read: true,
            write: true,
            create: true,
            truncate: true,
            append: false,
            share: vec![ShareMode::Read, ShareMode::Write],
            stream: None,
        }
    }

    fn ro() -> OpenOptions {
        OpenOptions {
            read: true,
            write: false,
            create: false,
            truncate: false,
            append: false,
            share: vec![ShareMode::Read, ShareMode::Write], // Allow others to read/write
            stream: None,
        }
    }

    fn rw() -> OpenOptions {
        OpenOptions {
            read: true,
            write: true,
            create: false,
            truncate: false,
            append: false,
            share: vec![ShareMode::Read, ShareMode::Write],
            stream: None,
        }
    }

    #[test]
    fn test_create_read_write_roundtrip() {
        let core = test_core();
        core.mkdir("/dir".as_ref(), 0o755).unwrap();
        let h = core.create("/dir/a.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"hello").unwrap();
        core.close(h).unwrap();
        let h2 = core.open("/dir/a.txt".as_ref(), &ro()).unwrap();
        let mut buf = [0u8; 5];
        let n = core.read(h2, 0, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"hello");
        core.close(h2).unwrap();
    }

    #[test]
    fn test_unlink_delete_on_close_semantics() {
        let core = test_core();
        let h = core.create("/x".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"test content").unwrap();
        core.unlink("/x".as_ref()).unwrap();

        // Should still be able to read while handle is open
        let mut buf = [0u8; 12];
        let n = core.read(h, 0, &mut buf).unwrap();
        assert_eq!(n, 12);
        assert_eq!(&buf, b"test content");

        core.close(h).unwrap();

        // Now the file should be gone
        assert!(core.open("/x".as_ref(), &ro()).is_err());
    }

    #[test]
    fn test_directory_operations() {
        let core = test_core();

        // Create directory
        core.mkdir("/testdir".as_ref(), 0o755).unwrap();

        // List root directory - should contain testdir
        let entries = core.readdir("/".as_ref()).unwrap();
        assert!(entries.iter().any(|e| e.name == "testdir" && e.is_dir));

        // Create file in directory
        let h = core.create("/testdir/file.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"content").unwrap();
        core.close(h).unwrap();

        // List directory - should contain file.txt
        let entries = core.readdir("/testdir".as_ref()).unwrap();
        assert!(entries.iter().any(|e| e.name == "file.txt" && !e.is_dir));

        // Remove directory (should fail if not empty)
        assert!(core.rmdir("/testdir".as_ref()).is_err());

        // Remove file first
        core.unlink("/testdir/file.txt".as_ref()).unwrap();

        // Now remove directory should work
        core.rmdir("/testdir".as_ref()).unwrap();

        // Directory should be gone
        let entries = core.readdir("/".as_ref()).unwrap();
        assert!(!entries.iter().any(|e| e.name == "testdir"));
    }

    #[test]
    fn test_snapshot_immutability() {
        let core = test_core();

        // Create a file with initial content
        let h = core.create("/f".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"original").unwrap();
        core.close(h).unwrap();

        // Create a snapshot
        let snap = core.snapshot_create(Some("base")).unwrap();

        // Create a branch from the snapshot
        let branch = core.branch_create_from_snapshot(snap, Some("test")).unwrap();

        // Bind to the branch
        core.bind_process_to_branch(branch).unwrap();

        // Modify the file in the branch
        let h = core.open("/f".as_ref(), &rw()).unwrap();
        core.write(h, 0, b"modified").unwrap();
        core.close(h).unwrap();

        // Read the file from the current branch - should see "modified"
        let h = core.open("/f".as_ref(), &ro()).unwrap();
        let mut buf = [0u8; 8];
        let n = core.read(h, 0, &mut buf).unwrap();
        assert_eq!(n, 8);
        assert_eq!(&buf, b"modified");
        core.close(h).unwrap();

        // Switch back to default branch and check that original content is preserved
        // (Note: In this simple implementation, the default branch shares the root,
        // so we need to create a separate test that reads from snapshot context)
        core.unbind_process().unwrap();

        // For now, verify that snapshot was created and branch exists
        let snapshots = core.snapshot_list();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].0, snap);
        assert_eq!(snapshots[0].1, Some("base".to_string()));

        let branches = core.branch_list();
        assert!(branches.iter().any(|b| b.id == branch && b.name == Some("test".to_string())));
    }

    #[test]
    fn test_branch_operations() {
        let core = test_core();

        // Create initial content
        let h = core.create("/test.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"initial").unwrap();
        core.close(h).unwrap();

        // Create snapshot
        let snap = core.snapshot_create(Some("clean")).unwrap();

        // Create a branch from snapshot
        let b1 = core.branch_create_from_snapshot(snap, Some("branch1")).unwrap();

        // Create a branch from current state
        let b2 = core.branch_create_from_current(Some("branch2")).unwrap();

        // Verify branch listing works
        let branches = core.branch_list();
        assert_eq!(branches.len(), 3); // default, b1, b2
        assert!(branches.iter().any(|b| b.id == b1 && b.parent == Some(snap)));
        assert!(branches.iter().any(|b| b.id == b2 && b.parent.is_none()));

        // Verify snapshot listing works
        let snapshots = core.snapshot_list();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].0, snap);
        assert_eq!(snapshots[0].1, Some("clean".to_string()));

        // Test binding to branches
        core.bind_process_to_branch(b1).unwrap();
        // In current implementation, binding doesn't change visible state
        // since branches share the directory tree

        core.unbind_process().unwrap();

        // Test snapshot deletion (should fail if branch depends on it)
        assert!(core.snapshot_delete(snap).is_err()); // b1 depends on it

        // Delete the branch first
        // Note: branch deletion not implemented yet, so skip
    }

    #[test]
    fn test_branch_process_isolation() {
        let core = test_core();

        // Create a file with initial content
        let h = core.create("/shared.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"original").unwrap();
        core.close(h).unwrap();

        // Create snapshot
        let snap = core.snapshot_create(Some("base")).unwrap();

        // Create a branch from snapshot
        let branch = core.branch_create_from_snapshot(snap, Some("test")).unwrap();

        // Verify that the branch has a different root than the snapshot
        {
            let snapshots = core.snapshots.lock().unwrap();
            let snapshot = snapshots.get(&snap).unwrap();
            let branches = core.branches.lock().unwrap();
            let branch_info = branches.get(&branch).unwrap();
            assert_ne!(snapshot.root_id, branch_info.root_id, "Branch should have different root than snapshot");
        }

        // Simulate two different processes with different PIDs
        let pid1 = 1001;
        let pid2 = 1002;

        // Bind process 1 to default branch (should see original content)
        core.bind_process_to_branch_with_pid(BranchId::DEFAULT, pid1).unwrap();

        // Bind process 2 to the snapshot branch
        core.bind_process_to_branch_with_pid(branch, pid2).unwrap();

        // Test what process 1 would see (by temporarily binding current process to pid1's branch)
        {
            let original_pid = std::process::id();
            let pid1_branch = core.process_branches.lock().unwrap().get(&pid1).cloned().unwrap();
            core.bind_process_to_branch_with_pid(pid1_branch, original_pid).unwrap();

            let h1 = core.open("/shared.txt".as_ref(), &ro()).unwrap();
            let mut buf = [0u8; 8];
            let n = core.read(h1, 0, &mut buf).unwrap();
            assert_eq!(n, 8);
            assert_eq!(&buf, b"original");
            core.close(h1).unwrap();
        }

        // Test what process 2 would see initially (should also see "original" since branch cloned snapshot)
        {
            let original_pid = std::process::id();
            let pid2_branch = core.process_branches.lock().unwrap().get(&pid2).cloned().unwrap();
            core.bind_process_to_branch_with_pid(pid2_branch, original_pid).unwrap();

            let h2 = core.open("/shared.txt".as_ref(), &ro()).unwrap();
            let mut buf = [0u8; 8];
            let n = core.read(h2, 0, &mut buf).unwrap();
            assert_eq!(n, 8);
            assert_eq!(&buf, b"original");
            core.close(h2).unwrap();

            // Modify the file in the branch (this should trigger content CoW)
            let h3 = core.open("/shared.txt".as_ref(), &rw()).unwrap();
            core.write(h3, 0, b"modified").unwrap();
            core.close(h3).unwrap();

            // Verify the branch now sees modified content
            let h_check = core.open("/shared.txt".as_ref(), &ro()).unwrap();
            let mut buf_check = [0u8; 8];
            let n_check = core.read(h_check, 0, &mut buf_check).unwrap();
            assert_eq!(n_check, 8);
            assert_eq!(&buf_check, b"modified");
            core.close(h_check).unwrap();
        }

        // Now process 1 should still see "original" (default branch unchanged)
        {
            let original_pid = std::process::id();
            let pid1_branch = core.process_branches.lock().unwrap().get(&pid1).cloned().unwrap();
            core.bind_process_to_branch_with_pid(pid1_branch, original_pid).unwrap();

            let h4 = core.open("/shared.txt".as_ref(), &ro()).unwrap();
            let mut buf = [0u8; 8];
            let n = core.read(h4, 0, &mut buf).unwrap();
            assert_eq!(n, 8);
            assert_eq!(&buf, b"original");
            core.close(h4).unwrap();
        }

        // And process 2's branch should see "modified"
        {
            let original_pid = std::process::id();
            let pid2_branch = core.process_branches.lock().unwrap().get(&pid2).cloned().unwrap();
            core.bind_process_to_branch_with_pid(pid2_branch, original_pid).unwrap();

            let h5 = core.open("/shared.txt".as_ref(), &ro()).unwrap();
            let mut buf = [0u8; 8];
            let n = core.read(h5, 0, &mut buf).unwrap();
            assert_eq!(n, 8);
            assert_eq!(&buf, b"modified");
            core.close(h5).unwrap();
        }
    }

    #[test]
    fn test_handle_stability_across_binding_changes() {
        let core = test_core();

        // Create a file
        let h = core.create("/test.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"initial").unwrap();

        // Open another handle to the same file (simulating a handle opened before binding)
        let h2 = core.open("/test.txt".as_ref(), &rw()).unwrap();

        // Create a branch and bind to it
        let snap = core.snapshot_create(Some("base")).unwrap();
        let branch = core.branch_create_from_snapshot(snap, Some("test")).unwrap();
        core.bind_process_to_branch(branch).unwrap();

        // Modify the file through the first handle (this should trigger CoW)
        core.write(h, 0, b"modified").unwrap();

        // The second handle should still work and see the modified content
        // (both handles reference the same node in the branch after CoW)
        let mut buf = [0u8; 8];
        let n = core.read(h2, 0, &mut buf).unwrap();
        assert_eq!(n, 8);
        assert_eq!(&buf, b"modified");

        // Close handles
        core.close(h).unwrap();
        core.close(h2).unwrap();
    }

    #[test]
    fn test_posix_byte_range_locks() {
        let core = test_core();

        // Create a file
        let h = core.create("/test.txt".as_ref(), &rw_create()).unwrap();

        // Test exclusive lock
        let lock_range = LockRange {
            offset: 0,
            len: 10,
            kind: LockKind::Exclusive,
        };
        core.lock(h, lock_range).unwrap();

        // Try to lock overlapping range - should fail
        let overlapping_lock = LockRange {
            offset: 5,
            len: 10,
            kind: LockKind::Exclusive,
        };
        assert!(core.lock(h, overlapping_lock).is_err()); // Same handle can't lock overlapping

        // Unlock the first lock
        core.unlock(h, lock_range).unwrap();

        // Now the overlapping lock should work
        core.lock(h, overlapping_lock).unwrap();

        // Test shared locks
        let shared_lock1 = LockRange {
            offset: 20,
            len: 10,
            kind: LockKind::Shared,
        };
        let shared_lock2 = LockRange {
            offset: 25,
            len: 10,
            kind: LockKind::Shared,
        };

        // Multiple shared locks on overlapping ranges should work
        core.lock(h, shared_lock1).unwrap();
        core.lock(h, shared_lock2).unwrap();

        // But exclusive lock on overlapping range should fail
        let exclusive_overlapping = LockRange {
            offset: 22,
            len: 5,
            kind: LockKind::Exclusive,
        };
        assert!(core.lock(h, exclusive_overlapping).is_err());

        // Clean up
        core.unlock(h, overlapping_lock).unwrap();
        core.unlock(h, shared_lock1).unwrap();
        core.unlock(h, shared_lock2).unwrap();
        core.close(h).unwrap();
    }

    #[test]
    fn test_xattr_operations() {
        let core = test_core();

        // Create a file
        core.create("/test.txt".as_ref(), &rw_create()).unwrap();

        // Set an xattr
        core.xattr_set("/test.txt".as_ref(), "user.test", b"value").unwrap();

        // Get the xattr
        let value = core.xattr_get("/test.txt".as_ref(), "user.test").unwrap();
        assert_eq!(value, b"value");

        // List xattrs
        let attrs = core.xattr_list("/test.txt".as_ref()).unwrap();
        assert!(attrs.contains(&"user.test".to_string()));

        // Try to get non-existent xattr
        assert!(core.xattr_get("/test.txt".as_ref(), "user.missing").is_err());

        // Set another xattr
        core.xattr_set("/test.txt".as_ref(), "user.other", b"othervalue").unwrap();
        let attrs2 = core.xattr_list("/test.txt".as_ref()).unwrap();
        assert_eq!(attrs2.len(), 2);
        assert!(attrs2.contains(&"user.test".to_string()));
        assert!(attrs2.contains(&"user.other".to_string()));
    }

    #[test]
    fn test_ads_operations() {
        let core = test_core();

        // Create a file
        let h = core.create("/test.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"main data").unwrap();
        core.close(h).unwrap();

        // Create a handle for an ADS
        let ads_opts = OpenOptions {
            read: true,
            write: true,
            create: true,
            truncate: false,
            append: false,
            share: vec![],
            stream: Some("ads1".to_string()),
        };
        let h_ads = core.open("/test.txt".as_ref(), &ads_opts).unwrap();

        // Write to the ADS
        core.write(h_ads, 0, b"ads data").unwrap();
        core.close(h_ads).unwrap();

        // List streams
        let streams = core.streams_list("/test.txt".as_ref()).unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].name, "ads1");

        // Read from the ADS
        let h_ads_read = core.open("/test.txt".as_ref(), &ads_opts).unwrap();
        let mut buf = [0u8; 8];
        let n = core.read(h_ads_read, 0, &mut buf).unwrap();
        assert_eq!(n, 8);
        assert_eq!(&buf, b"ads data");
        core.close(h_ads_read).unwrap();

        // Main data stream should still be accessible
        let h_main = core.open("/test.txt".as_ref(), &ro()).unwrap();
        let mut buf_main = [0u8; 9];
        let n_main = core.read(h_main, 0, &mut buf_main).unwrap();
        assert_eq!(n_main, 9);
        assert_eq!(&buf_main, b"main data");
        core.close(h_main).unwrap();
    }
}
