//! Filesystem snapshot provider abstractions for Agents Workflow.
//!
//! This crate provides abstractions for different filesystem snapshot technologies
//! (ZFS, Btrfs, etc.) to enable time travel capabilities in the AW system.

use async_trait::async_trait;
use std::path::Path;

pub mod error;

/// Result type for filesystem snapshot operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for filesystem snapshot operations.
pub use error::Error;

/// Core trait for filesystem snapshot providers.
#[async_trait]
pub trait FsSnapshotProvider: Send + Sync {
    /// Create a workspace at the destination path.
    ///
    /// This method should create a snapshot of the current working directory
    /// and make it available at the specified destination path.
    async fn create_workspace(&self, dest: &Path) -> Result<()>;

    /// Clean up a workspace at the destination path.
    ///
    /// This method should remove the workspace and any associated snapshots.
    async fn cleanup_workspace(&self, dest: &Path) -> Result<()>;
}

/// Auto-detect and return the appropriate provider for a given path.
pub fn provider_for(_path: &Path) -> Result<Box<dyn FsSnapshotProvider>> {
    // For now, always return the copy provider as a fallback
    // TODO: Implement proper provider detection when subcrates are integrated
    Ok(Box::new(CopyProvider))
}

/// Copy-based provider as a fallback when no advanced snapshot filesystem is available.
pub struct CopyProvider;

#[async_trait]
impl FsSnapshotProvider for CopyProvider {
    async fn create_workspace(&self, dest: &Path) -> Result<()> {
        if dest.exists() {
            return Err(Error::DestinationExists {
                path: dest.to_path_buf(),
            });
        }

        // For now, just create the directory as a placeholder
        tokio::fs::create_dir_all(dest).await?;
        Ok(())
    }

    async fn cleanup_workspace(&self, dest: &Path) -> Result<()> {
        if dest.exists() {
            tokio::fs::remove_dir_all(dest).await?;
        }
        Ok(())
    }
}
