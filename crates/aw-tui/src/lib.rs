//! Terminal User Interface for Agents-Workflow
//!
//! This crate provides a Ratatui-based TUI for creating, monitoring,
//! and managing agent coding sessions with seamless multiplexer integration.

pub mod app;
pub mod ui;
pub mod event;
pub mod error;
pub mod msg;
pub mod model;
pub mod viewmodel;
pub mod test_runtime;

pub use app::*;
pub use error::*;
pub use msg::*;
pub use model::*;
pub use viewmodel::*;
pub use test_runtime::*;

use ratatui::{backend::TestBackend, Terminal};

/// Helpers for tests/runners to render with a deterministic backend
pub fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("test terminal")
}
