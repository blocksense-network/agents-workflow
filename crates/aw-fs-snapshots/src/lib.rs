//! Filesystem snapshot provider abstractions for Agents Workflow.
//!
//! This crate provides abstractions for different filesystem snapshot technologies
//! (ZFS, Btrfs, etc.) to enable time travel capabilities in the AW system.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod error;

/// Result type for filesystem snapshot operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for filesystem snapshot operations.
pub use error::Error;

/// Provider kinds for filesystem snapshot operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotProviderKind {
    /// Auto-detect the best provider for the given path.
    Auto,
    /// ZFS snapshot provider.
    Zfs,
    /// Btrfs snapshot provider.
    Btrfs,
    /// AgentFS user-space filesystem provider.
    AgentFs,
    /// Git-based snapshot provider.
    Git,
    /// Disable snapshotting entirely.
    Disable,
}

/// Working copy modes for prepared workspaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkingCopyMode {
    /// Auto-detect the best working copy mode.
    Auto,
    /// Use copy-on-write overlay mounting to preserve path stability.
    CowOverlay,
    /// Use a separate worktree directory.
    Worktree,
    /// Execute directly in the original working copy (no isolation).
    InPlace,
}

/// Capabilities detected for a provider on a given repository path.
#[derive(Clone, Debug)]
pub struct ProviderCapabilities {
    /// The kind of provider.
    pub kind: SnapshotProviderKind,
    /// Capability score (higher is better, 0-100).
    pub score: u8,
    /// Whether this provider supports copy-on-write overlay mode.
    pub supports_cow_overlay: bool,
    /// Detection notes shown in diagnostics.
    pub notes: Vec<String>,
}

/// A prepared workspace ready for agent execution.
#[derive(Clone, Debug)]
pub struct PreparedWorkspace {
    /// Path where agent processes should run.
    pub exec_path: PathBuf,
    /// The working copy mode used.
    pub working_copy: WorkingCopyMode,
    /// The provider kind used.
    pub provider: SnapshotProviderKind,
    /// Opaque handle for idempotent teardown across crashes/process boundaries.
    pub cleanup_token: String,
}

/// Reference to a snapshot created by a provider.
#[derive(Clone, Debug)]
pub struct SnapshotRef {
    /// Provider-opaque snapshot identifier.
    pub id: String,
    /// Optional user-visible label.
    pub label: Option<String>,
    /// Provider kind that created this snapshot.
    pub provider: SnapshotProviderKind,
    /// Additional metadata.
    pub meta: HashMap<String, String>,
}

/// Core trait for filesystem snapshot providers.
#[async_trait]
pub trait FsSnapshotProvider: Send + Sync {
    /// Return the kind of this provider.
    fn kind(&self) -> SnapshotProviderKind;

    /// Detect capabilities for the current host/repo.
    fn detect_capabilities(&self, repo: &Path) -> ProviderCapabilities;

    /// Create a session workspace (independent or in-place) for the selected working-copy mode.
    async fn prepare_writable_workspace(
        &self,
        repo: &Path,
        mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace>;

    /// Snapshot current workspace state; label is optional UI hint.
    async fn snapshot_now(&self, ws: &PreparedWorkspace, label: Option<&str>) -> Result<SnapshotRef>;

    /// Read-only inspection mount for a snapshot (optional).
    async fn mount_readonly(&self, snap: &SnapshotRef) -> Result<PathBuf>;

    /// Create a new writable workspace (branch) from a snapshot.
    async fn branch_from_snapshot(
        &self,
        snap: &SnapshotRef,
        mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace>;

    /// Cleanup/destroy any resources created by this provider (workspaces, mounts).
    async fn cleanup(&self, token: &str) -> Result<()>;
}

/// Auto-detect and return the appropriate provider for a given path.
pub fn provider_for(path: &Path) -> Result<Box<dyn FsSnapshotProvider>> {
    // Validate the path first
    validate_destination_path(path)?;

    // Detect available providers in order of preference
    let providers: Vec<Box<dyn FsSnapshotProvider>> = vec![
        #[cfg(feature = "zfs")]
        Box::new(crate::zfs::ZfsProvider::new()),
        #[cfg(feature = "btrfs")]
        Box::new(crate::btrfs::BtrfsProvider::new()),
        Box::new(CopyProvider::new()),
    ];

    // Find the provider with the highest capability score
    let mut best_provider = None;
    let mut best_score = 0;

    for provider in providers {
        let capabilities = provider.detect_capabilities(path);
        if capabilities.score > best_score {
            best_score = capabilities.score;
            best_provider = Some(provider);
        }
    }

    best_provider.ok_or_else(|| Error::provider("No suitable provider found"))
}

/// Validate a destination path for workspace creation.
fn validate_destination_path(dest: &Path) -> Result<()> {
    // Check if the destination path can be created as a directory
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            // Try to create parent directory to validate permissions
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::provider(format!("Cannot create parent directory {}: {}", parent.display(), e))
            })?;
            // Clean up the test directory
            if parent.exists() && std::fs::read_dir(parent).map_or(true, |mut d| d.next().is_none()) {
                let _ = std::fs::remove_dir(parent);
            }
        }
    }

    // Ensure it's not trying to create in system directories
    let invalid_paths = ["/dev", "/proc", "/sys", "/run"];
    for invalid in &invalid_paths {
        if dest.starts_with(invalid) {
            return Err(Error::provider(format!("Cannot create workspace in system directory: {}", dest.display())));
        }
    }

    // Check for other invalid paths
    if dest == Path::new("/") {
        return Err(Error::provider("Cannot create workspace at root directory"));
    }

    Ok(())
}

/// Copy-based provider as a fallback when no advanced snapshot filesystem is available.
pub struct CopyProvider {
    _private: (),
}

impl CopyProvider {
    /// Create a new copy provider.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for CopyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FsSnapshotProvider for CopyProvider {
    fn kind(&self) -> SnapshotProviderKind {
        SnapshotProviderKind::Git // Copy provider acts as Git fallback
    }

    fn detect_capabilities(&self, _repo: &Path) -> ProviderCapabilities {
        ProviderCapabilities {
            kind: self.kind(),
            score: 10, // Low priority fallback
            supports_cow_overlay: false,
            notes: vec!["Copy-based provider - no snapshot isolation".to_string()],
        }
    }

    async fn prepare_writable_workspace(
        &self,
        repo: &Path,
        mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        match mode {
            WorkingCopyMode::InPlace => {
                // For in-place mode, just return the repo path directly
                Ok(PreparedWorkspace {
                    exec_path: repo.to_path_buf(),
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("copy:inplace:{}", repo.display()),
                })
            }
            WorkingCopyMode::Worktree => {
                // For worktree mode, create a copy of the repo
                let exec_path = repo.with_extension("copy");
                if exec_path.exists() {
                    return Err(Error::DestinationExists {
                        path: exec_path,
                    });
                }

                // Copy the repository
                tokio::process::Command::new("cp")
                    .arg("-a")
                    .arg("--reflink=auto")
                    .arg(repo)
                    .arg(&exec_path)
                    .status()
                    .await
                    .map_err(|e| Error::provider(format!("Copy command failed: {}", e)))?
                    .success()
                    .then_some(())
                    .ok_or_else(|| Error::provider("Copy command failed"))?;

                Ok(PreparedWorkspace {
                    exec_path,
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("copy:worktree:{}", repo.display()),
                })
            }
            WorkingCopyMode::CowOverlay | WorkingCopyMode::Auto => {
                // Fall back to worktree for copy provider
                self.prepare_writable_workspace(repo, WorkingCopyMode::Worktree).await
            }
        }
    }

    async fn snapshot_now(&self, _ws: &PreparedWorkspace, _label: Option<&str>) -> Result<SnapshotRef> {
        // Copy provider doesn't support snapshots
        Err(Error::provider("Copy provider does not support snapshots"))
    }

    async fn mount_readonly(&self, _snap: &SnapshotRef) -> Result<PathBuf> {
        // Copy provider doesn't support snapshots
        Err(Error::provider("Copy provider does not support readonly mounting"))
    }

    async fn branch_from_snapshot(
        &self,
        _snap: &SnapshotRef,
        _mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        // Copy provider doesn't support snapshots
        Err(Error::provider("Copy provider does not support branching from snapshots"))
    }

    async fn cleanup(&self, token: &str) -> Result<()> {
        if token.starts_with("copy:inplace:") {
            // Nothing to cleanup for in-place mode
            Ok(())
        } else if token.starts_with("copy:worktree:") {
            // Extract the path and remove it
            let path_str = token.strip_prefix("copy:worktree:").unwrap_or(token);
            let path = Path::new(path_str);
            if path.exists() {
                tokio::fs::remove_dir_all(path).await?;
            }
            Ok(())
        } else {
            Err(Error::provider(format!("Invalid cleanup token: {}", token)))
        }
    }
}

#[cfg(feature = "zfs")]
pub mod zfs {
    include!("../zfs_stub.rs");
}

#[cfg(feature = "btrfs")]
pub mod btrfs {
    include!("../btrfs_stub.rs");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_path_validation() {
        // Valid paths should pass validation
        assert!(validate_destination_path(Path::new("/tmp/test")).is_ok());
        assert!(validate_destination_path(Path::new("./test")).is_ok());

        // Invalid paths should be rejected
        assert!(validate_destination_path(Path::new("/")).is_err());
        assert!(validate_destination_path(Path::new("/dev/null")).is_err());
        assert!(validate_destination_path(Path::new("/proc/version")).is_err());
        assert!(validate_destination_path(Path::new("/sys/class")).is_err());
        assert!(validate_destination_path(Path::new("/run/lock")).is_err());
    }

    #[test]
    fn test_copy_provider_capabilities() {
        let provider = CopyProvider::new();
        let capabilities = provider.detect_capabilities(Path::new("/tmp"));

        assert_eq!(capabilities.kind, SnapshotProviderKind::Git);
        assert_eq!(capabilities.score, 10);
        assert!(!capabilities.supports_cow_overlay);
        assert!(!capabilities.notes.is_empty());
    }

    #[test]
    fn test_copy_provider_prepare_inplace_workspace() {
        let provider = CopyProvider::new();
        let repo = Path::new("/tmp/test_repo");

        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            provider.prepare_writable_workspace(repo, WorkingCopyMode::InPlace)
        );

        assert!(result.is_ok());
        let ws = result.unwrap();
        assert_eq!(ws.exec_path, repo);
        assert_eq!(ws.working_copy, WorkingCopyMode::InPlace);
        assert_eq!(ws.provider, SnapshotProviderKind::Git);
        assert!(ws.cleanup_token.starts_with("copy:inplace:"));
    }

    #[test]
    fn test_copy_provider_cleanup() {
        let provider = CopyProvider::new();

        // Test inplace cleanup (should be no-op)
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            provider.cleanup("copy:inplace:/some/path")
        );
        assert!(result.is_ok());

        // Test invalid token
        let result = tokio::runtime::Runtime::new().unwrap().block_on(
            provider.cleanup("invalid:token")
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_for_fallback() {
        // Since we don't have ZFS/Btrfs available in tests, it should fall back to CopyProvider
        let result = provider_for(Path::new("/tmp"));
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.kind(), SnapshotProviderKind::Git);
    }
}
