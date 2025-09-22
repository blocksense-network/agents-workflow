//! Core type definitions for AgentFS

use serde::{Deserialize, Serialize};

/// Opaque snapshot identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub [u8; 16]);

impl SnapshotId {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Opaque branch identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchId(pub [u8; 16]);

impl BranchId {
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Opaque handle identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandleId(pub u64);

impl HandleId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

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

/// File open options
#[derive(Clone, Debug)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
    pub append: bool,
    pub share: Vec<ShareMode>,
    pub stream: Option<String>,
}

/// Share mode for Windows compatibility
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShareMode {
    Read,
    Write,
    Delete,
}

/// Lock kind for byte-range locking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockKind {
    Shared,
    Exclusive,
}

/// Byte range lock specification
#[derive(Clone, Copy, Debug)]
pub struct LockRange {
    pub offset: u64,
    pub len: u64,
    pub kind: LockKind,
}

/// Fallocate mode (optional operation)
#[derive(Clone, Copy, Debug)]
pub enum FallocateMode {
    Allocate,
    PunchHole,
}

/// Content identifier for storage backend
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentId(pub u64);

impl ContentId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}
