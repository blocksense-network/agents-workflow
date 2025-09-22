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
            share: vec![],
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
            share: vec![],
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
}
