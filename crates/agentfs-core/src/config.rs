//! Configuration types for AgentFS Core

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Case sensitivity modes
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CaseSensitivity {
    Sensitive,
    InsensitivePreserving,
}

/// Memory policy for storage backends
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryPolicy {
    pub max_bytes_in_memory: Option<u64>,
    pub spill_directory: Option<PathBuf>,
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self {
            max_bytes_in_memory: Some(1024 * 1024 * 1024), // 1GB
            spill_directory: None,
        }
    }
}

/// System limits
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FsLimits {
    pub max_open_handles: u32,
    pub max_branches: u32,
    pub max_snapshots: u32,
}

impl Default for FsLimits {
    fn default() -> Self {
        Self {
            max_open_handles: 10000,
            max_branches: 1000,
            max_snapshots: 10000,
        }
    }
}

/// Cache policy settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachePolicy {
    pub attr_ttl_ms: u32,
    pub entry_ttl_ms: u32,
    pub negative_ttl_ms: u32,
    pub enable_readdir_plus: bool,
    pub auto_cache: bool,
    pub writeback_cache: bool,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            attr_ttl_ms: 1000,
            entry_ttl_ms: 1000,
            negative_ttl_ms: 1000,
            enable_readdir_plus: true,
            auto_cache: true,
            writeback_cache: false,
        }
    }
}

/// Main filesystem configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FsConfig {
    pub case_sensitivity: CaseSensitivity,
    pub memory: MemoryPolicy,
    pub limits: FsLimits,
    pub cache: CachePolicy,
    pub enable_xattrs: bool,
    pub enable_ads: bool,
    pub track_events: bool,
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            case_sensitivity: CaseSensitivity::Sensitive,
            memory: MemoryPolicy::default(),
            limits: FsLimits::default(),
            cache: CachePolicy::default(),
            enable_xattrs: true,
            enable_ads: false,
            track_events: false,
        }
    }
}
