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
}
