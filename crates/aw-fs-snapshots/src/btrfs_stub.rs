//! Stub implementation for Btrfs provider when the btrfs feature is not enabled.

use crate::*;

/// Stub Btrfs provider that returns an error when used without the btrfs feature.
pub struct BtrfsProvider;

impl BtrfsProvider {
    /// Create a new Btrfs provider stub.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl FsSnapshotProvider for BtrfsProvider {
    fn kind(&self) -> SnapshotProviderKind {
        SnapshotProviderKind::Btrfs
    }

    fn detect_capabilities(&self, _repo: &Path) -> ProviderCapabilities {
        ProviderCapabilities {
            kind: self.kind(),
            score: 0,
            supports_cow_overlay: false,
            notes: vec!["Btrfs support not compiled in".to_string()],
        }
    }

    async fn prepare_writable_workspace(
        &self,
        _repo: &Path,
        _mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        Err(Error::provider("Btrfs support not compiled in - enable the 'btrfs' feature"))
    }

    async fn snapshot_now(&self, _ws: &PreparedWorkspace, _label: Option<&str>) -> Result<SnapshotRef> {
        Err(Error::provider("Btrfs support not compiled in - enable the 'btrfs' feature"))
    }

    async fn mount_readonly(&self, _snap: &SnapshotRef) -> Result<PathBuf> {
        Err(Error::provider("Btrfs support not compiled in - enable the 'btrfs' feature"))
    }

    async fn branch_from_snapshot(
        &self,
        _snap: &SnapshotRef,
        _mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        Err(Error::provider("Btrfs support not compiled in - enable the 'btrfs' feature"))
    }

    async fn cleanup(&self, _token: &str) -> Result<()> {
        Err(Error::provider("Btrfs support not compiled in - enable the 'btrfs' feature"))
    }
}
