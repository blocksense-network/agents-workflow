//! AgentFS Protocol — Control plane types and validation
//!
//! This crate defines the SSZ schemas and request/response types
//! for the AgentFS control plane, used by CLI tools and adapters.

pub mod messages;
pub mod validation;

// Re-export key types
pub use messages::{
    BranchBindRequest, BranchBindResponse, BranchCreateRequest, BranchCreateResponse, BranchInfo,
    ErrorResponse, Request, Response, SnapshotCreateRequest, SnapshotCreateResponse, SnapshotInfo,
    SnapshotListRequest, SnapshotListResponse,
};
pub use validation::*;
