//! Core error types for the AW system.

/// Core error type for all AW operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Task error: {message}")]
    Task { message: String },

    #[error("Session error: {message}")]
    Session { message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    /// Create a new task-related error.
    pub fn task<S: Into<String>>(message: S) -> Self {
        Self::Task {
            message: message.into(),
        }
    }

    /// Create a new session-related error.
    pub fn session<S: Into<String>>(message: S) -> Self {
        Self::Session {
            message: message.into(),
        }
    }

    /// Create a new generic error.
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic(message.into())
    }
}
