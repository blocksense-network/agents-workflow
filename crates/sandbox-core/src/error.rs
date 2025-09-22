//! Error types for the sandbox core module.

use thiserror::Error;

/// Errors that can occur in sandbox operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Namespace creation failed: {0}")]
    Namespace(String),

    #[error("Process execution failed: {0}")]
    Execution(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
