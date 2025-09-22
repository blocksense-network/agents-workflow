//! Core task and session lifecycle orchestration for Agents Workflow.
//!
//! This crate provides the foundational abstractions and orchestration logic for
//! managing agent tasks and sessions, including lifecycle management, state
//! transitions, and coordination with other AW components.

pub mod error;
pub mod task;
pub mod session;

/// Core result type used throughout the AW system.
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type that encompasses all AW operations.
pub use error::Error;

/// Task lifecycle management and orchestration.
pub use task::{Task, TaskId, TaskStatus, TaskManager};

/// Session lifecycle management and orchestration.
pub use session::{Session, SessionId, SessionStatus, SessionManager};
