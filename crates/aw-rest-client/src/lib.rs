//! REST API client for Agents-Workflow service
//!
//! This crate provides a complete HTTP client for the Agents-Workflow REST API
//! as specified in REST-Service.md. It includes support for authentication,
//! request/response handling, and SSE streaming for real-time updates.

pub mod client;
pub mod auth;
pub mod error;
pub mod sse;

pub use client::*;
pub use auth::*;
pub use error::*;
