//! Core task and session lifecycle orchestration for Agents Workflow.
//!
//! This crate provides the foundational abstractions and orchestration logic for
//! managing agent tasks and sessions, including lifecycle management, state
//! transitions, and coordination with other AW components.

pub mod agent_tasks;
pub mod editor;
pub mod error;
pub mod session;
pub mod task;

/// Core result type used throughout the AW system.
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type that encompasses all AW operations.
pub use error::Error;

/// Task lifecycle management and orchestration.
pub use task::{Task, TaskId, TaskManager, TaskStatus};

/// Session lifecycle management and orchestration.
pub use session::{Session, SessionId, SessionManager, SessionStatus};

/// Agent task file management and operations.
pub use agent_tasks::AgentTasks;

/// Interactive editor integration for task content creation.
pub use editor::{edit_content_interactive, EditorError, EDITOR_HINT};
