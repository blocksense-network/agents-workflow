//! AgentFS Protocol — Control plane types and validation
//!
//! This crate defines the JSON schemas and request/response types
//! for the AgentFS control plane, used by CLI tools and adapters.

pub mod messages;
pub mod validation;

// Re-export key types
pub use messages::*;
