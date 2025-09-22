//! AgentFS FFI — C ABI for FSKit and other integrations
//!
//! This crate provides a C-compatible ABI for integrating AgentFS
//! with platform-specific filesystem frameworks like FSKit (macOS).

pub mod c_api;

// Re-export C API functions
pub use c_api::*;
