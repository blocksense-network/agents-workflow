//! Error types for cgroup operations.

use thiserror::Error;

/// Errors that can occur in cgroup operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cgroup setup failed: {0}")]
    Setup(String),

    #[error("Limit enforcement failed: {0}")]
    Limit(String),

    #[error("Metrics collection failed: {0}")]
    Metrics(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}
