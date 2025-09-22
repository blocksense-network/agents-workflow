//! ZFS snapshot provider implementation for Agents Workflow.

use async_trait::async_trait;
use aw_fs_snapshots::{FsSnapshotProvider, ProviderCapabilities, PreparedWorkspace, Result, SnapshotProviderKind, SnapshotRef, WorkingCopyMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// ZFS snapshot provider implementation.
#[derive(Default)]
pub struct ZfsProvider;

impl ZfsProvider {
    /// Create a new ZFS provider.
    pub fn new() -> Self {
        Self
    }

    /// Check if ZFS is available on this system.
    fn zfs_available() -> bool {
        // Only available on Linux
        if !cfg!(target_os = "linux") {
            return false;
        }

        // Check if zfs command exists
        std::process::Command::new("which")
            .arg("zfs")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get the filesystem type for a given path.
    fn fs_type(path: &Path) -> Result<String> {
        let output = std::process::Command::new("stat")
            .args(["-f", "-c", "%T"])
            .arg(path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()?;

        if !output.status.success() {
            return Err(aw_fs_snapshots::Error::provider("Failed to determine filesystem type"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the ZFS dataset for a given path.
    fn get_dataset_for_path(&self, path: &Path) -> Result<String> {
        let output = std::process::Command::new("zfs")
            .args(["list", "-H", "-o", "name,mountpoint"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()?;

        if !output.status.success() {
            return Err(aw_fs_snapshots::Error::provider("Failed to list ZFS datasets"));
        }

        let datasets: Vec<(String, String)> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    let (name, mount) = (parts[0].to_string(), parts[1].to_string());
                    // Filter out non-mounted datasets and root
                    if mount != "none" && mount != "legacy" && mount != "/" && path.starts_with(&mount) {
                        Some((name, mount))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Find the dataset with the longest mountpoint that contains the path
        let best_match = datasets
            .into_iter()
            .filter(|(_, mount)| path.starts_with(Path::new(mount)))
            .max_by_key(|(_, mount)| mount.len());

        match best_match {
            Some((dataset, _)) => Ok(dataset),
            None => Err(aw_fs_snapshots::Error::UnsupportedFilesystem {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Execute a ZFS command.
    async fn execute_zfs_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("zfs")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(aw_fs_snapshots::Error::provider(format!(
                "ZFS command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Generate a unique identifier for ZFS resources.
    fn generate_unique_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("aw_{}_{}", std::process::id(), timestamp)
    }
}

#[async_trait]
impl FsSnapshotProvider for ZfsProvider {
    fn kind(&self) -> SnapshotProviderKind {
        SnapshotProviderKind::Zfs
    }

    fn detect_capabilities(&self, repo: &Path) -> ProviderCapabilities {
        if !Self::zfs_available() {
            return ProviderCapabilities {
                kind: self.kind(),
                score: 0,
                supports_cow_overlay: false,
                notes: vec!["ZFS command not available".to_string()],
            };
        }

        match Self::fs_type(repo) {
            Ok(fs_type) if fs_type == "zfs" => {
                match self.get_dataset_for_path(repo) {
                    Ok(dataset) => ProviderCapabilities {
                        kind: self.kind(),
                        score: 90,
                        supports_cow_overlay: true,
                        notes: vec![format!("Using ZFS dataset: {}", dataset)],
                    },
                    Err(_) => ProviderCapabilities {
                        kind: self.kind(),
                        score: 0,
                        supports_cow_overlay: false,
                        notes: vec!["No ZFS dataset found for path".to_string()],
                    },
                }
            }
            Ok(fs_type) => ProviderCapabilities {
                kind: self.kind(),
                score: 0,
                supports_cow_overlay: false,
                notes: vec![format!("Path is on {} filesystem, not ZFS", fs_type)],
            },
            Err(e) => ProviderCapabilities {
                kind: self.kind(),
                score: 0,
                supports_cow_overlay: false,
                notes: vec![format!("Failed to detect filesystem: {}", e)],
            },
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
                    cleanup_token: format!("zfs:inplace:{}", repo.display()),
                })
            }
            WorkingCopyMode::CowOverlay => {
                // ZFS CoW overlay mode: create snapshot + clone
                let dataset = self.get_dataset_for_path(repo)?;
                let unique_id = self.generate_unique_id();
                let snapshot_name = format!("{}@aw_snapshot_{}", dataset, unique_id);
                let clone_name = format!("{}-aw_clone_{}", dataset, unique_id);

                // Create snapshot
                self.execute_zfs_command(&["snapshot", &snapshot_name]).await?;

                // Create clone (will be mounted automatically)
                self.execute_zfs_command(&["clone", &snapshot_name, &clone_name]).await?;

                // Get mountpoint of the clone
                let mountpoint = self.execute_zfs_command(&["get", "-H", "-o", "value", "mountpoint", &clone_name]).await?;

                let exec_path = if mountpoint == "none" || mountpoint == "legacy" {
                    // Clone not auto-mounted, find where it should be
                    return Err(aw_fs_snapshots::Error::provider(
                        "ZFS clone not mounted - manual mounting not yet implemented"
                    ));
                } else {
                    PathBuf::from(mountpoint)
                };

                Ok(PreparedWorkspace {
                    exec_path: exec_path.clone(),
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("zfs:cow:{}:{}", snapshot_name, clone_name),
                })
            }
            WorkingCopyMode::Worktree | WorkingCopyMode::Auto => {
                // Fall back to worktree mode for ZFS (simpler implementation)
                // In practice, ZFS would typically use CoW overlay
                Err(aw_fs_snapshots::Error::provider("ZFS worktree mode not implemented - use CowOverlay"))
            }
        }
    }

    async fn snapshot_now(&self, ws: &PreparedWorkspace, label: Option<&str>) -> Result<SnapshotRef> {
        let dataset = self.get_dataset_for_path(&ws.exec_path)?;
        let unique_id = self.generate_unique_id();
        let snapshot_name = format!("{}@aw_session_{}", dataset, unique_id);

        self.execute_zfs_command(&["snapshot", &snapshot_name]).await?;

        let mut meta = HashMap::new();
        meta.insert("dataset".to_string(), dataset.clone());
        meta.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(SnapshotRef {
            id: snapshot_name,
            label: label.map(|s| s.to_string()),
            provider: self.kind(),
            meta,
        })
    }

    async fn mount_readonly(&self, snap: &SnapshotRef) -> Result<PathBuf> {
        // For ZFS, snapshots are typically accessed by mounting the snapshot directly
        // This is a simplified implementation
        let snapshot_path = format!("{}/.zfs/snapshot/{}", snap.meta.get("dataset").unwrap_or(&"".to_string()), snap.id.split('@').next_back().unwrap_or(""));
        let mount_path = PathBuf::from(snapshot_path);

        if mount_path.exists() {
            Ok(mount_path)
        } else {
            Err(aw_fs_snapshots::Error::provider("ZFS snapshot not accessible via .zfs directory"))
        }
    }

    async fn branch_from_snapshot(
        &self,
        snap: &SnapshotRef,
        mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        match mode {
            WorkingCopyMode::CowOverlay => {
                let unique_id = self.generate_unique_id();
                let clone_name = format!("{}-aw_branch_{}", snap.meta.get("dataset").unwrap_or(&"".to_string()), unique_id);

                // Create clone from the snapshot
                self.execute_zfs_command(&["clone", &snap.id, &clone_name]).await?;

                // Get mountpoint
                let mountpoint = self.execute_zfs_command(&["get", "-H", "-o", "value", "mountpoint", &clone_name]).await?;

                let exec_path = if mountpoint == "none" || mountpoint == "legacy" {
                    return Err(aw_fs_snapshots::Error::provider(
                        "ZFS clone not mounted - manual mounting not yet implemented"
                    ));
                } else {
                    PathBuf::from(mountpoint)
                };

                Ok(PreparedWorkspace {
                    exec_path,
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("zfs:branch:{}", clone_name),
                })
            }
            _ => Err(aw_fs_snapshots::Error::provider("ZFS branching only supports CowOverlay mode")),
        }
    }

    async fn cleanup(&self, token: &str) -> Result<()> {
        if token.starts_with("zfs:inplace:") {
            // Nothing to cleanup for in-place mode
            Ok(())
        } else if token.starts_with("zfs:cow:") {
            // Format: zfs:cow:snapshot_name:clone_name
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() >= 4 {
                let snapshot = parts[2];
                let clone = parts[3];

                // Destroy clone first, then snapshot
                let _ = self.execute_zfs_command(&["destroy", clone]).await;
                let _ = self.execute_zfs_command(&["destroy", snapshot]).await;
            }
            Ok(())
        } else if token.starts_with("zfs:branch:") {
            // Format: zfs:branch:clone_name
            let clone = token.strip_prefix("zfs:branch:").unwrap_or(token);
            let _ = self.execute_zfs_command(&["destroy", clone]).await;
            Ok(())
        } else {
            Err(aw_fs_snapshots::Error::provider(format!("Invalid ZFS cleanup token: {}", token)))
        }
    }
}
