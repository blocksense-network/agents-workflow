//! Control plane message types for AgentFS

use agentfs_core::{BranchId, SnapshotId};
use serde::{Deserialize, Serialize};

/// Base message envelope
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub version: String,
    #[serde(flatten)]
    pub payload: T,
}

/// Snapshot creation request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotCreateRequest {
    pub op: String, // "snapshot.create"
    pub name: Option<String>,
}

/// Snapshot creation response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotCreateResponse {
    pub snapshot_id: SnapshotId,
}

/// Snapshot list request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotListRequest {
    pub op: String, // "snapshot.list"
}

/// Snapshot list response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotListResponse {
    pub snapshots: Vec<SnapshotInfo>,
}

/// Snapshot information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: SnapshotId,
    pub name: Option<String>,
}

/// Branch creation request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchCreateRequest {
    pub op: String, // "branch.create"
    pub from_snapshot: SnapshotId,
    pub name: Option<String>,
}

/// Branch creation response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchCreateResponse {
    pub branch_id: BranchId,
}

/// Branch bind request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchBindRequest {
    pub op: String, // "branch.bind"
    pub branch_id: BranchId,
    pub pid: Option<u32>, // defaults to current process
}

/// Branch bind response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchBindResponse {
    // Empty on success
}

/// Generic error response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: Option<i32>,
}
