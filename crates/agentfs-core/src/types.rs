//! Core type definitions for AgentFS

use serde::{Deserialize, Serialize};

/// Opaque snapshot identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId([u8; 16]);

/// Opaque branch identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId([u8; 16]);

/// Opaque handle identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandleId(u64);

/// File timestamps
#[derive(Clone, Copy, Debug)]
pub struct FileTimes {
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub birthtime: i64,
}

/// File permissions
#[derive(Clone, Debug)]
pub struct FileMode {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
}

/// File attributes
#[derive(Clone, Debug)]
pub struct Attributes {
    pub len: u64,
    pub times: FileTimes,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub mode_user: FileMode,
    pub mode_group: FileMode,
    pub mode_other: FileMode,
}

/// Directory entry information
#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub len: u64,
}

/// Extended attribute entry
#[derive(Clone, Debug)]
pub struct XattrEntry {
    pub name: String,
    pub value: Vec<u8>,
}

/// Stream specification (for ADS)
#[derive(Clone, Debug)]
pub struct StreamSpec {
    pub name: String,
}
