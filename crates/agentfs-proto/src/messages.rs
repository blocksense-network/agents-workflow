//! Control plane message types for AgentFS

use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};

// SSZ Union-based request/response types for type-safe communication
// Using Vec<u8> for strings as SSZ supports variable-length byte vectors

/// Request union - each variant contains version and operation-specific data
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[ssz(enum_behaviour = "union")]
pub enum Request {
    SnapshotCreate((Vec<u8>, SnapshotCreateRequest)), // (version, request)
    SnapshotList(Vec<u8>),                            // version
    BranchCreate((Vec<u8>, BranchCreateRequest)),     // (version, request)
    BranchBind((Vec<u8>, BranchBindRequest)),         // (version, request)
}

/// Response union - operation-specific success responses or errors
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
#[ssz(enum_behaviour = "union")]
pub enum Response {
    SnapshotCreate(SnapshotCreateResponse),
    SnapshotList(SnapshotListResponse),
    BranchCreate(BranchCreateResponse),
    BranchBind(BranchBindResponse),
    Error(ErrorResponse),
}

/// Error response
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct ErrorResponse {
    pub error: Vec<u8>,
    pub code: Option<u32>,
}

/// Snapshot creation request payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SnapshotCreateRequest {
    pub name: Option<Vec<u8>>,
}

/// Snapshot creation response payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SnapshotCreateResponse {
    pub snapshot: SnapshotInfo,
}

/// Snapshot list request payload (empty)
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SnapshotListRequest {}

/// Snapshot list response payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SnapshotListResponse {
    pub snapshots: Vec<SnapshotInfo>,
}

/// Snapshot information
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SnapshotInfo {
    pub id: Vec<u8>,
    pub name: Option<Vec<u8>>,
}

/// Branch creation request payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct BranchCreateRequest {
    pub from: Vec<u8>,
    pub name: Option<Vec<u8>>,
}

/// Branch creation response payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct BranchCreateResponse {
    pub branch: BranchInfo,
}

/// Branch information
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct BranchInfo {
    pub id: Vec<u8>,
    pub name: Option<Vec<u8>>,
    pub parent: Vec<u8>,
}

/// Branch bind request payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct BranchBindRequest {
    pub branch: Vec<u8>,
    pub pid: Option<u32>,
}

/// Branch bind response payload
#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct BranchBindResponse {
    pub branch: Vec<u8>,
    pub pid: u32,
}

// Constructors for SSZ union variants (convert String to Vec<u8>)
impl Request {
    pub fn snapshot_create(name: Option<String>) -> Self {
        Self::SnapshotCreate((
            b"1".to_vec(),
            SnapshotCreateRequest {
                name: name.map(|s| s.into_bytes()),
            },
        ))
    }

    pub fn snapshot_list() -> Self {
        Self::SnapshotList(b"1".to_vec())
    }

    pub fn branch_create(from: String, name: Option<String>) -> Self {
        Self::BranchCreate((
            b"1".to_vec(),
            BranchCreateRequest {
                from: from.into_bytes(),
                name: name.map(|s| s.into_bytes()),
            },
        ))
    }

    pub fn branch_bind(branch: String, pid: Option<u32>) -> Self {
        Self::BranchBind((
            b"1".to_vec(),
            BranchBindRequest {
                branch: branch.into_bytes(),
                pid,
            },
        ))
    }
}

impl Response {
    pub fn snapshot_create(snapshot: SnapshotInfo) -> Self {
        Self::SnapshotCreate(SnapshotCreateResponse { snapshot })
    }

    pub fn snapshot_list(snapshots: Vec<SnapshotInfo>) -> Self {
        Self::SnapshotList(SnapshotListResponse { snapshots })
    }

    pub fn branch_create(branch: BranchInfo) -> Self {
        Self::BranchCreate(BranchCreateResponse { branch })
    }

    pub fn branch_bind(branch: Vec<u8>, pid: u32) -> Self {
        Self::BranchBind(BranchBindResponse { branch, pid })
    }

    pub fn error(message: String, code: Option<u32>) -> Self {
        Self::Error(ErrorResponse {
            error: message.into_bytes(),
            code,
        })
    }
}
