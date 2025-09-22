//! Configuration types for AgentFS Core

use std::path::PathBuf;

/// Case sensitivity modes
#[derive(Clone, Copy, Debug)]
pub enum CaseSensitivity {
    Sensitive,
    InsensitivePreserving,
}

/// Memory policy for storage backends
#[derive(Clone, Debug)]
pub struct MemoryPolicy {
    pub max_bytes_in_memory: Option<u64>,
    pub spill_directory: Option<PathBuf>,
}

/// System limits
#[derive(Clone, Debug)]
pub struct FsLimits {
    pub max_open_handles: u32,
    pub max_branches: u32,
    pub max_snapshots: u32,
}

/// Cache policy settings
#[derive(Clone, Debug)]
pub struct CachePolicy {
    pub attr_ttl_ms: u32,
    pub entry_ttl_ms: u32,
    pub negative_ttl_ms: u32,
    pub enable_readdir_plus: bool,
    pub auto_cache: bool,
    pub writeback_cache: bool,
}

/// Main filesystem configuration
#[derive(Clone, Debug)]
pub struct FsConfig {
    pub case_sensitivity: CaseSensitivity,
    pub memory: MemoryPolicy,
    pub limits: FsLimits,
    pub cache: CachePolicy,
    pub enable_xattrs: bool,
    pub enable_ads: bool,
    pub track_events: bool,
}
