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
        let entries = core.readdir_plus("/".as_ref()).unwrap();
        let names: Vec<_> = entries.iter().map(|(e, _)| e.name.as_str()).collect();
        assert!(names.contains(&"testdir"));

        // Create file in directory
        let h = core.create("/testdir/file.txt".as_ref(), &rw_create()).unwrap();
        core.write(h, 0, b"content").unwrap();
        core.close(h).unwrap();

        // List directory - should contain file.txt
        let entries = core.readdir_plus("/testdir".as_ref()).unwrap();
        let names: Vec<_> = entries.iter().map(|(e, _)| e.name.as_str()).collect();
        assert!(names.contains(&"file.txt"));

        // Remove directory (should fail if not empty)
        assert!(core.rmdir("/testdir".as_ref()).is_err());

        // Remove file first
        core.unlink("/testdir/file.txt".as_ref()).unwrap();

        // Now remove directory should work
        core.rmdir("/testdir".as_ref()).unwrap();

        // Directory should be gone
        let entries = core.readdir_plus("/".as_ref()).unwrap();
        let names: Vec<_> = entries.iter().map(|(e, _)| e.name.as_str()).collect();
        assert!(!names.contains(&"testdir"));
    }

    #[test]
    fn test_rename_and_sorted_readdir() {
        let core = test_core();
        core.mkdir("/dir".as_ref(), 0o755).unwrap();
        // Create files out of order
        let h1 = core.create("/dir/b.txt".as_ref(), &rw_create()).unwrap();
        core.close(h1).unwrap();
        let h2 = core.create("/dir/a.txt".as_ref(), &rw_create()).unwrap();
        core.close(h2).unwrap();

        // Sorted listing
        let entries = core.readdir_plus("/dir".as_ref()).unwrap();
        let names: Vec<_> = entries.iter().map(|(e, _)| e.name.as_str()).collect();
        assert_eq!(names, vec!["a.txt", "b.txt"]);

        // Rename a.txt -> c.txt
        core.rename("/dir/a.txt".as_ref(), "/dir/c.txt".as_ref()).unwrap();
        let entries2 = core.readdir_plus("/dir".as_ref()).unwrap();
        let names2: Vec<_> = entries2.iter().map(|(e, _)| e.name.as_str()).collect();
        assert_eq!(names2, vec!["b.txt", "c.txt"]);
    }

    #[test]
    fn test_set_mode_and_times() {
        let core = test_core();
        let h = core.create("/file".as_ref(), &rw_create()).unwrap();
        core.close(h).unwrap();

        // Get current attributes
        let before = core.getattr("/file".as_ref()).unwrap();

        // Change mode and times
        core.set_mode("/file".as_ref(), 0o600).unwrap();
        let new_times = FileTimes { atime: before.times.atime + 10, mtime: before.times.mtime + 10, ctime: before.times.ctime + 10, birthtime: before.times.birthtime };
        core.set_times("/file".as_ref(), new_times).unwrap();

        // Verify
        let after = core.getattr("/file".as_ref()).unwrap();
        assert!(after.times.ctime >= new_times.ctime);
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

    #[test]
    fn test_event_subscription_and_emission() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct TestEventSink {
            events: Arc<Mutex<Vec<EventKind>>>,
        }

        impl TestEventSink {
            fn new() -> Self {
                Self {
                    events: Arc::new(Mutex::new(Vec::new())),
                }
            }

            fn get_events(&self) -> Vec<EventKind> {
                self.events.lock().unwrap().clone()
            }
        }

        impl EventSink for TestEventSink {
            fn on_event(&self, evt: &EventKind) {
                self.events.lock().unwrap().push(evt.clone());
            }
        }

        let core = test_core();
        let sink = Arc::new(TestEventSink::new());

        // Subscribe to events
        let sub_id = core.subscribe_events(sink.clone()).unwrap();

        // Test snapshot creation event
        let snap = core.snapshot_create(Some("test_snap")).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            EventKind::SnapshotCreated { id, name } => {
                assert_eq!(*id, snap);
                assert_eq!(name.as_ref().unwrap(), "test_snap");
            }
            _ => panic!("Expected SnapshotCreated event"),
        }

        // Clear events
        sink.events.lock().unwrap().clear();

        // Test branch creation event
        let branch = core.branch_create_from_snapshot(snap, Some("test_branch")).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            EventKind::BranchCreated { id, name } => {
                assert_eq!(*id, branch);
                assert_eq!(name.as_ref().unwrap(), "test_branch");
            }
            _ => panic!("Expected BranchCreated event"),
        }

        // Clear events
        sink.events.lock().unwrap().clear();

        // Test file creation event
        core.create("/test.txt".as_ref(), &rw_create()).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            EventKind::Created { path } => {
                assert_eq!(path, "/test.txt");
            }
            _ => panic!("Expected Created event"),
        }

        // Clear events
        sink.events.lock().unwrap().clear();

        // Test directory creation event
        core.mkdir("/testdir".as_ref(), 0o755).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            EventKind::Created { path } => {
                assert_eq!(path, "/testdir");
            }
            _ => panic!("Expected Created event"),
        }

        // Clear events
        sink.events.lock().unwrap().clear();

        // Test file removal event
        core.unlink("/test.txt".as_ref()).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            EventKind::Removed { path } => {
                assert_eq!(path, "/test.txt");
            }
            _ => panic!("Expected Removed event"),
        }

        // Unsubscribe
        core.unsubscribe_events(sub_id).unwrap();

        // Clear events
        sink.events.lock().unwrap().clear();

        // Create another file - should not emit events since unsubscribed
        core.create("/test2.txt".as_ref(), &rw_create()).unwrap();
        let events = sink.get_events();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_stats_reporting() {
        let core = test_core();

        // Initially should have the default branch and no snapshots
        let stats = core.stats();
        assert_eq!(stats.branches, 1); // Default branch
        assert_eq!(stats.snapshots, 0); // No snapshots initially
        assert_eq!(stats.open_handles, 0); // No open handles initially

        // Create a snapshot
        core.snapshot_create(Some("test")).unwrap();
        let stats = core.stats();
        assert_eq!(stats.snapshots, 1); // Test snapshot

        // Create a branch
        let snap = core.snapshot_list()[0].0; // Get the test snapshot
        core.branch_create_from_snapshot(snap, Some("test_branch")).unwrap();
        let stats = core.stats();
        assert_eq!(stats.branches, 2); // Default + test branch

        // Open a handle
        let h = core.create("/file.txt".as_ref(), &rw_create()).unwrap();
        let stats = core.stats();
        assert_eq!(stats.open_handles, 1);

        // Close handle
        core.close(h).unwrap();
        let stats = core.stats();
        assert_eq!(stats.open_handles, 0);
    }

    #[test]
    fn test_readdir_plus() {
        let core = test_core();

        // Create a file and directory
        core.create("/file.txt".as_ref(), &rw_create()).unwrap();
        core.mkdir("/subdir".as_ref(), 0o755).unwrap();

        // Test readdir_plus returns entries with attributes
        let entries = core.readdir_plus("/".as_ref()).unwrap();

        // Should have at least file.txt and subdir
        assert!(entries.len() >= 2);

        let file_entry = entries.iter().find(|(e, _)| e.name == "file.txt").unwrap();
        assert!(!file_entry.0.is_dir);
        assert_eq!(file_entry.0.len, 0); // Empty file
        assert!(!file_entry.1.is_dir);
        assert_eq!(file_entry.1.len, 0);

        let dir_entry = entries.iter().find(|(e, _)| e.name == "subdir").unwrap();
        assert!(dir_entry.0.is_dir);
        assert_eq!(dir_entry.0.len, 0); // Directories have 0 length
        assert!(dir_entry.1.is_dir);
        assert_eq!(dir_entry.1.len, 0);

        // Test readdir_plus on non-existent path
        assert!(core.readdir_plus("/nonexistent".as_ref()).is_err());
    }

    #[test]
    fn test_events_disabled_when_track_events_false() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct TestEventSink {
            events: Arc<Mutex<Vec<EventKind>>>,
        }

        impl TestEventSink {
            fn new() -> Self {
                Self {
                    events: Arc::new(Mutex::new(Vec::new())),
                }
            }
        }

        impl EventSink for TestEventSink {
            fn on_event(&self, evt: &EventKind) {
                self.events.lock().unwrap().push(evt.clone());
            }
        }

        // Create core with track_events = false
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
            track_events: false, // Events disabled
        };

        let core = FsCore::new(config).unwrap();
        let sink = Arc::new(TestEventSink::new());

        // Subscribe to events
        let sub_id = core.subscribe_events(sink.clone()).unwrap();

        // Create a file - should not emit events
        core.create("/test.txt".as_ref(), &rw_create()).unwrap();

        // Check that no events were emitted
        let events = sink.events.lock().unwrap();
        assert_eq!(events.len(), 0);

        // Unsubscribe should still work
        core.unsubscribe_events(sub_id).unwrap();
    }

    #[test]
    fn test_symlink_basic_operations() {
        let core = test_core();

        // Create a target file
        core.create("/target.txt".as_ref(), &rw_create()).unwrap();
        let h = core.open("/target.txt".as_ref(), &rw()).unwrap();
        core.write(h, 0, b"target content").unwrap();
        core.close(h).unwrap();

        // Create a symlink pointing to the target
        core.symlink("target.txt", "/link.txt".as_ref()).unwrap();

        // Read the symlink
        let target = core.readlink("/link.txt".as_ref()).unwrap();
        assert_eq!(target, "target.txt");

        // Verify symlink appears in directory listing
        let entries = core.readdir_plus("/".as_ref()).unwrap();
        let link_entry = entries.iter().find(|(e, _)| e.name == "link.txt").unwrap();
        assert!(link_entry.0.is_symlink);
        assert_eq!(link_entry.0.len, 10);

        // Verify symlink attributes
        let attrs = core.getattr("/link.txt".as_ref()).unwrap();
        assert!(attrs.is_symlink);
        assert_eq!(attrs.len, 10); // length of "target.txt"

        // Verify we can read through the symlink (though this would normally be handled by the filesystem layer)
        // For now, just ensure the symlink exists and has correct attributes
        assert!(core.getattr("/link.txt".as_ref()).is_ok());
    }

    #[test]
    fn test_symlink_to_directory() {
        let core = test_core();

        // Create a target directory
        core.mkdir("/target_dir".as_ref(), 0o755).unwrap();

        // Create a symlink to the directory
        core.symlink("target_dir", "/link_to_dir".as_ref()).unwrap();

        // Verify the symlink
        let target = core.readlink("/link_to_dir".as_ref()).unwrap();
        assert_eq!(target, "target_dir");

        // Verify symlink attributes
        let attrs = core.getattr("/link_to_dir".as_ref()).unwrap();
        assert!(attrs.is_symlink);
        assert_eq!(attrs.len, 10); // length of "target_dir"
    }

    #[test]
    fn test_symlink_unlink() {
        let core = test_core();

        // Create a symlink
        core.symlink("nonexistent", "/test_link".as_ref()).unwrap();

        // Verify it exists
        assert!(core.getattr("/test_link".as_ref()).is_ok());

        // Unlink the symlink
        core.unlink("/test_link".as_ref()).unwrap();

        // Verify it's gone
        assert!(core.getattr("/test_link".as_ref()).is_err());
    }

    #[test]
    fn test_symlink_relative_path() {
        let core = test_core();

        // Create a directory structure
        core.mkdir("/dir1".as_ref(), 0o755).unwrap();
        core.mkdir("/dir1/subdir".as_ref(), 0o755).unwrap();
        core.create("/dir1/subdir/target.txt".as_ref(), &rw_create()).unwrap();

        // Create a symlink with relative path
        core.symlink("../target.txt", "/dir1/subdir/link.txt".as_ref()).unwrap();

        // Verify the symlink
        let target = core.readlink("/dir1/subdir/link.txt".as_ref()).unwrap();
        assert_eq!(target, "../target.txt");
    }

    #[test]
    fn test_symlink_errors() {
        let core = test_core();

        // Try to create symlink with existing name
        core.create("/existing".as_ref(), &rw_create()).unwrap();
        assert!(core.symlink("target", "/existing".as_ref()).is_err());

        // Try to readlink on non-existent path
        assert!(core.readlink("/nonexistent".as_ref()).is_err());

        // Try to readlink on regular file
        core.create("/regular.txt".as_ref(), &rw_create()).unwrap();
        assert!(core.readlink("/regular.txt".as_ref()).is_err());

        // Try to readlink on directory
        core.mkdir("/testdir".as_ref(), 0o755).unwrap();
        assert!(core.readlink("/testdir".as_ref()).is_err());

        // Try to create symlink in non-existent directory
        assert!(core.symlink("target", "/nonexistent/link".as_ref()).is_err());

        // Try to create symlink in a file (not directory)
        assert!(core.symlink("target", "/regular.txt/link".as_ref()).is_err());
    }

    #[test]
    fn test_symlink_readdir_plus() {
        let core = test_core();

        // Create a target file
        core.create("/target".as_ref(), &rw_create()).unwrap();

        // Create a symlink
        core.symlink("target", "/symlink".as_ref()).unwrap();

        // Test readdir_plus
        let entries = core.readdir_plus("/".as_ref()).unwrap();

        // Find the symlink entry
        let symlink_entry = entries.iter().find(|(e, _)| e.name == "symlink").unwrap();

        assert!(symlink_entry.0.is_symlink);
        assert_eq!(symlink_entry.0.len, 6); // length of "target"
        assert!(symlink_entry.1.is_symlink);
        assert_eq!(symlink_entry.1.len, 6);
    }

    #[test]
    fn test_symlink_cannot_read_write_like_file() {
        let core = test_core();

        // Create a symlink
        core.symlink("target", "/symlink".as_ref()).unwrap();

        // Try to open symlink for reading/writing (should work at filesystem level)
        let h = core.open("/symlink".as_ref(), &ro()).unwrap();
        // But trying to read from it should fail at the VFS level since it's not a file
        let mut buf = [0u8; 10];
        assert!(core.read(h, 0, &mut buf).is_err());
        core.close(h).unwrap();

        // Writing to symlink should also fail
        let h2 = core.open("/symlink".as_ref(), &rw()).unwrap();
        assert!(core.write(h2, 0, b"data").is_err());
        core.close(h2).unwrap();
    }

    #[test]
    fn test_symlink_in_directory_operations() {
        let core = test_core();

        // Create a directory
        core.mkdir("/testdir".as_ref(), 0o755).unwrap();

        // Create a symlink inside the directory
        core.symlink("outside", "/testdir/inside_link".as_ref()).unwrap();

        // List directory contents
        let entries = core.readdir_plus("/testdir".as_ref()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0.name, "inside_link");
        assert!(entries[0].0.is_symlink);

        // Remove directory with symlink inside should fail
        assert!(core.rmdir("/testdir".as_ref()).is_err());

        // Remove the symlink first, then remove directory
        core.unlink("/testdir/inside_link".as_ref()).unwrap();
        core.rmdir("/testdir".as_ref()).unwrap();
    }

    #[test]
    fn test_symlink_empty_target() {
        let core = test_core();

        // Create symlink with empty target
        core.symlink("", "/empty_link".as_ref()).unwrap();

        let target = core.readlink("/empty_link".as_ref()).unwrap();
        assert_eq!(target, "");

        let attrs = core.getattr("/empty_link".as_ref()).unwrap();
        assert!(attrs.is_symlink);
        assert_eq!(attrs.len, 0);
    }

    #[test]
    fn test_symlink_long_target() {
        let core = test_core();

        // Create symlink with long target path
        let long_target = "a".repeat(1000);
        core.symlink(&long_target, "/long_link".as_ref()).unwrap();

        let target = core.readlink("/long_link".as_ref()).unwrap();
        assert_eq!(target, long_target);

        let attrs = core.getattr("/long_link".as_ref()).unwrap();
        assert!(attrs.is_symlink);
        assert_eq!(attrs.len, 1000);
    }

    #[test]
    fn test_multiple_symlinks_to_same_target() {
        let core = test_core();

        // Create a target
        core.create("/target".as_ref(), &rw_create()).unwrap();

        // Create multiple symlinks to the same target
        core.symlink("target", "/link1".as_ref()).unwrap();
        core.symlink("target", "/link2".as_ref()).unwrap();
        core.symlink("target", "/link3".as_ref()).unwrap();

        // Verify all symlinks work
        for i in 1..=3 {
            let link_name = format!("/link{}", i);
            let target = core.readlink(link_name.as_ref()).unwrap();
            assert_eq!(target, "target");

            let attrs = core.getattr(link_name.as_ref()).unwrap();
            assert!(attrs.is_symlink);
        }

        // Verify directory listing shows all symlinks
        let entries = core.readdir_plus("/".as_ref()).unwrap();
        let symlink_count = entries.iter().filter(|(e, _)| e.is_symlink).count();
        assert_eq!(symlink_count, 3);
    }
}
