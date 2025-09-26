//! Terminal User Interface for Agents-Workflow
//!
//! This crate provides a Ratatui-based TUI for creating, monitoring,
//! and managing agent coding sessions with seamless multiplexer integration.

pub mod app;
pub mod ui;
pub mod event;
pub mod error;

pub use app::*;
pub use error::*;
