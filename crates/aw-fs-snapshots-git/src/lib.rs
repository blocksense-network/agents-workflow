//! Git-based filesystem snapshot provider for Agents Workflow.
//!
//! This provider implements filesystem snapshots using Git's native capabilities,
//! providing a portable fallback for environments without native CoW filesystems
//! like ZFS or Btrfs. It uses shadow repositories with object sharing and
//! git worktrees for efficient workspace management.

use async_trait::async_trait;
use aw_fs_snapshots_traits::{FsSnapshotProvider, ProviderCapabilities, PreparedWorkspace, Result, SnapshotProviderKind, SnapshotRef, WorkingCopyMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Git-based snapshot provider implementation.
pub struct GitProvider {
    /// Base directory for shadow repositories (defaults to ~/.cache/aw/git-shadows)
    shadow_repo_dir: PathBuf,
    /// Base directory for worktrees (defaults to ~/.cache/aw/git-worktrees)
    worktree_dir: PathBuf,
    /// Whether to include untracked files in snapshots (defaults to false)
    include_untracked: bool,
}

impl GitProvider {
    /// Create a new Git provider with default configuration.
    pub fn new() -> Self {
        Self {
            shadow_repo_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("aw")
                .join("git-shadows"),
            worktree_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("aw")
                .join("git-worktrees"),
            include_untracked: false,
        }
    }

    /// Create a new Git provider with custom configuration.
    pub fn with_config(shadow_repo_dir: PathBuf, worktree_dir: PathBuf, include_untracked: bool) -> Self {
        Self {
            shadow_repo_dir,
            worktree_dir,
            include_untracked,
        }
    }

    /// Check if Git is available on this system.
    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if the given path is a Git repository.
    fn is_git_repo(path: &Path) -> bool {
        std::process::Command::new("git")
            .args(["-C", &path.to_string_lossy(), "rev-parse", "--git-dir"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get the path to the shadow repository for a given primary repository.
    fn shadow_repo_path(&self, primary_repo: &Path) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        primary_repo.canonicalize().unwrap_or_else(|_| primary_repo.to_path_buf()).hash(&mut hasher);
        let hash = hasher.finish();

        self.shadow_repo_dir.join(format!("{:x}", hash))
    }

    /// Get the path for a worktree for a given session and branch.
    fn worktree_path(&self, session_id: &str, branch_id: &str) -> PathBuf {
        self.worktree_dir.join(format!("{}_{}", session_id, branch_id))
    }

    /// Ensure the shadow repository exists and is properly configured.
    async fn ensure_shadow_repo(&self, primary_repo: &Path) -> Result<PathBuf> {
        let shadow_path = self.shadow_repo_path(primary_repo);

        if !shadow_path.exists() {
            // Create shadow repository
            tokio::fs::create_dir_all(&shadow_path).await
                .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create shadow repo directory: {}", e)))?;

            // Initialize bare repository
            let status = Command::new("git")
                .args(["init", "--bare"])
                .arg(&shadow_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
                .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to init shadow repo: {}", e)))?;

            if !status.success() {
                return Err(aw_fs_snapshots_traits::Error::provider("Failed to initialize shadow repository"));
            }

            // Configure shadow repository
            let config_commands = vec![
                vec!["config", "gc.auto", "0"],
                vec!["config", "receive.denyCurrentBranch", "ignore"],
            ];

            for args in config_commands {
                let status = Command::new("git")
                    .args(&args)
                    .current_dir(&shadow_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await
                    .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to configure shadow repo: {}", e)))?;

                if !status.success() {
                    return Err(aw_fs_snapshots_traits::Error::provider("Failed to configure shadow repository"));
                }
            }

            // Add alternates to share objects with primary repo
            let primary_git_dir = primary_repo.join(".git");
            let alternates_file = shadow_path.join("objects").join("info").join("alternates");

            tokio::fs::create_dir_all(alternates_file.parent().unwrap()).await
                .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create alternates dir: {}", e)))?;

            let alternates_content = format!("{}\n", primary_git_dir.join("objects").display());
            tokio::fs::write(&alternates_file, alternates_content).await
                .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to write alternates file: {}", e)))?;
        }

        Ok(shadow_path)
    }

    /// Generate a unique identifier for resources.
    fn generate_unique_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("aw_git_{}_{}", std::process::id(), timestamp)
    }

    /// Create a snapshot commit in the shadow repository.
    async fn create_snapshot_commit(&self, primary_repo: &Path, shadow_repo: &Path, label: Option<&str>) -> Result<String> {
        // Create a temporary index for staging changes
        let temp_index = tempfile::NamedTempFile::new()
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create temp index: {}", e)))?;
        let temp_index_path = temp_index.path();
        let primary_repo_str = primary_repo.to_string_lossy();

        // Get the current HEAD commit from primary repo
        let head_commit = Command::new("git")
            .args(["-C", &primary_repo_str, "rev-parse", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to get HEAD commit: {}", e)))?;

        if !head_commit.status.success() {
            return Err(aw_fs_snapshots_traits::Error::provider("Failed to get HEAD commit from primary repository"));
        }

        let head_commit = String::from_utf8_lossy(&head_commit.stdout).trim().to_string();

        // Stage all changes using git add
        let mut add_args = vec!["-C", &primary_repo_str, "add", "--all"];
        if self.include_untracked {
            add_args.push("--no-ignore-removal");
        }

        let status = Command::new("git")
            .args(&add_args)
            .env("GIT_INDEX_FILE", temp_index_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to stage changes: {}", e)))?;

        if !status.success() {
            return Err(aw_fs_snapshots_traits::Error::provider("Failed to stage changes for snapshot"));
        }

        // Create commit message
        let commit_message = match label {
            Some(label) => format!("AW Snapshot: {}", label),
            None => "AW Snapshot".to_string(),
        };

        // Create commit using git commit-tree
        let mut commit_tree_args = vec![
            "commit-tree",
            "-m", &commit_message,
            "-p", &head_commit,
        ];

        // Read the tree from the temporary index
        let tree_output = Command::new("git")
            .args(["write-tree"])
            .env("GIT_INDEX_FILE", temp_index_path)
            .current_dir(primary_repo)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to write tree: {}", e)))?;

        if !tree_output.status.success() {
            return Err(aw_fs_snapshots_traits::Error::provider("Failed to write tree from index"));
        }

        let tree_hash = String::from_utf8_lossy(&tree_output.stdout).trim().to_string();
        commit_tree_args.push(&tree_hash);

        let commit_output = Command::new("git")
            .args(&commit_tree_args)
            .current_dir(shadow_repo)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create commit: {}", e)))?;

        if !commit_output.status.success() {
            return Err(aw_fs_snapshots_traits::Error::provider("Failed to create snapshot commit"));
        }

        Ok(String::from_utf8_lossy(&commit_output.stdout).trim().to_string())
    }
}

#[async_trait]
impl FsSnapshotProvider for GitProvider {
    fn kind(&self) -> SnapshotProviderKind {
        SnapshotProviderKind::Git
    }

    fn detect_capabilities(&self, repo: &Path) -> ProviderCapabilities {
        if !Self::git_available() {
            return ProviderCapabilities {
                kind: self.kind(),
                score: 0,
                supports_cow_overlay: false,
                notes: vec!["Git command not available".to_string()],
            };
        }

        if !Self::is_git_repo(repo) {
            return ProviderCapabilities {
                kind: self.kind(),
                score: 0,
                supports_cow_overlay: false,
                notes: vec!["Path is not a Git repository".to_string()],
            };
        }

        // Git provider has a moderate score as a fallback
        ProviderCapabilities {
            kind: self.kind(),
            score: 10,
            supports_cow_overlay: false, // Git doesn't support true CoW
            notes: vec![
                "Git-based snapshots available".to_string(),
                format!("Shadow repo: {}", self.shadow_repo_path(repo).display()),
            ],
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
                    cleanup_token: format!("git:inplace:{}", repo.display()),
                })
            }
            WorkingCopyMode::Worktree => {
                // Create a git worktree for isolated workspace
                let session_id = self.generate_unique_id();
                let branch_name = format!("aw-worktree-{}", session_id);
                let worktree_path = self.worktree_path(&session_id, "main");

                // Create worktree
                let status = Command::new("git")
                    .args([
                        "-C", &repo.to_string_lossy(),
                        "worktree", "add",
                        "--detach",
                        &worktree_path.to_string_lossy(),
                        "HEAD"
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await
                    .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create worktree: {}", e)))?;

                if !status.success() {
                    return Err(aw_fs_snapshots_traits::Error::provider("Failed to create git worktree"));
                }

                Ok(PreparedWorkspace {
                    exec_path: worktree_path,
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("git:worktree:{}:{}", session_id, branch_name),
                })
            }
            WorkingCopyMode::CowOverlay | WorkingCopyMode::Auto => {
                // Fall back to worktree mode for Git
                Box::pin(self.prepare_writable_workspace(repo, WorkingCopyMode::Worktree)).await
            }
        }
    }

    async fn snapshot_now(&self, ws: &PreparedWorkspace, label: Option<&str>) -> Result<SnapshotRef> {
        let shadow_repo = self.ensure_shadow_repo(&ws.exec_path).await?;
        let commit_hash = self.create_snapshot_commit(&ws.exec_path, &shadow_repo, label).await?;

        let mut meta = HashMap::new();
        meta.insert("shadow_repo".to_string(), shadow_repo.to_string_lossy().to_string());
        meta.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        Ok(SnapshotRef {
            id: commit_hash,
            label: label.map(|s| s.to_string()),
            provider: self.kind(),
            meta,
        })
    }

    async fn mount_readonly(&self, snap: &SnapshotRef) -> Result<PathBuf> {
        // For Git, create a temporary worktree at the snapshot commit
        let shadow_repo = PathBuf::from(snap.meta.get("shadow_repo").as_ref()
            .ok_or_else(|| aw_fs_snapshots_traits::Error::provider("Missing shadow_repo in snapshot metadata"))?);

        let session_id = self.generate_unique_id();
        let worktree_path = self.worktree_path(&session_id, "readonly");

        let status = Command::new("git")
            .args([
                "worktree", "add",
                "--detach",
                &worktree_path.to_string_lossy(),
                &snap.id
            ])
            .current_dir(&shadow_repo)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create readonly worktree: {}", e)))?;

        if !status.success() {
            return Err(aw_fs_snapshots_traits::Error::provider("Failed to create readonly worktree for snapshot"));
        }

        Ok(worktree_path)
    }

    async fn branch_from_snapshot(
        &self,
        snap: &SnapshotRef,
        mode: WorkingCopyMode,
    ) -> Result<PreparedWorkspace> {
        match mode {
            WorkingCopyMode::Worktree => {
                let shadow_repo = PathBuf::from(snap.meta.get("shadow_repo").as_ref()
                    .ok_or_else(|| aw_fs_snapshots_traits::Error::provider("Missing shadow_repo in snapshot metadata"))?);

                let session_id = self.generate_unique_id();
                let branch_name = format!("aw-branch-{}", session_id);
                let worktree_path = self.worktree_path(&session_id, &branch_name);

                // Create worktree from snapshot commit
                let status = Command::new("git")
                    .args([
                        "worktree", "add",
                        "--detach",
                        &worktree_path.to_string_lossy(),
                        &snap.id
                    ])
                    .current_dir(&shadow_repo)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await
                    .map_err(|e| aw_fs_snapshots_traits::Error::provider(format!("Failed to create branch worktree: {}", e)))?;

                if !status.success() {
                    return Err(aw_fs_snapshots_traits::Error::provider("Failed to create worktree for branch"));
                }

                Ok(PreparedWorkspace {
                    exec_path: worktree_path,
                    working_copy: mode,
                    provider: self.kind(),
                    cleanup_token: format!("git:branch:{}:{}", session_id, branch_name),
                })
            }
            _ => Err(aw_fs_snapshots_traits::Error::provider("Git branching only supports Worktree mode")),
        }
    }

    async fn cleanup(&self, token: &str) -> Result<()> {
        if token.starts_with("git:inplace:") {
            // Nothing to cleanup for in-place mode
            Ok(())
        } else if token.starts_with("git:worktree:") {
            // Format: git:worktree:session_id:branch_name
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() >= 4 {
                let session_id = parts[2];
                let branch_name = parts[3];
                let worktree_path = self.worktree_path(session_id, branch_name);

                // Remove worktree
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force"])
                    .arg(&worktree_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await;

                // Remove worktree directory if it still exists
                let _ = tokio::fs::remove_dir_all(&worktree_path).await;
            }
            Ok(())
        } else if token.starts_with("git:branch:") {
            // Format: git:branch:session_id:branch_name
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() >= 4 {
                let session_id = parts[2];
                let branch_name = parts[3];
                let worktree_path = self.worktree_path(session_id, branch_name);

                // Remove worktree
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force"])
                    .arg(&worktree_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await;

                // Remove worktree directory if it still exists
                let _ = tokio::fs::remove_dir_all(&worktree_path).await;
            }
            Ok(())
        } else {
            Err(aw_fs_snapshots_traits::Error::provider(format!("Invalid Git cleanup token: {}", token)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_provider_creation() {
        let provider = GitProvider::new();
        assert_eq!(provider.kind(), SnapshotProviderKind::Git);
    }

    #[tokio::test]
    async fn test_git_capabilities_on_non_git_path() {
        let provider = GitProvider::new();
        let capabilities = provider.detect_capabilities(Path::new("/tmp"));

        // On a non-git path, should have low score
        assert_eq!(capabilities.kind, SnapshotProviderKind::Git);
        assert_eq!(capabilities.score, 0);
        assert!(!capabilities.supports_cow_overlay);
    }

    #[tokio::test]
    async fn test_git_inplace_workspace_creation() {
        let provider = GitProvider::new();
        let repo_path = Path::new("/tmp/test_repo");

        let result = provider.prepare_writable_workspace(repo_path, WorkingCopyMode::InPlace).await;

        // Should succeed even without git repo
        assert!(result.is_ok());
        let ws = result.unwrap();
        assert_eq!(ws.working_copy, WorkingCopyMode::InPlace);
        assert_eq!(ws.provider, SnapshotProviderKind::Git);
        assert!(ws.cleanup_token.starts_with("git:inplace:"));
    }

    #[tokio::test]
    async fn test_git_worktree_mode_not_implemented() {
        let provider = GitProvider::new();
        let repo_path = Path::new("/tmp/test_repo");

        let result = provider.prepare_writable_workspace(repo_path, WorkingCopyMode::Worktree).await;

        // Should fail without a real git repo
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_auto_mode_falls_back_to_worktree() {
        let provider = GitProvider::new();
        let repo_path = Path::new("/tmp/test_repo");

        let result = provider.prepare_writable_workspace(repo_path, WorkingCopyMode::Auto).await;

        // Should fail (same as worktree without real repo)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_invalid_token() {
        let provider = GitProvider::new();
        let result = provider.cleanup("invalid:token").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Git cleanup token"));
    }

    #[tokio::test]
    async fn test_cleanup_inplace_token() {
        let provider = GitProvider::new();
        let result = provider.cleanup("git:inplace:/some/path").await;

        // Should succeed (no-op)
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_unique_id() {
        let provider = GitProvider::new();
        let id1 = provider.generate_unique_id();
        let id2 = provider.generate_unique_id();

        // IDs should be different
        assert_ne!(id1, id2);

        // Should contain process ID
        let pid = std::process::id().to_string();
        assert!(id1.contains(&pid));
    }

    #[tokio::test]
    async fn test_shadow_repo_path_deterministic() {
        let provider = GitProvider::new();
        let repo_path = Path::new("/tmp/test_repo");

        let path1 = provider.shadow_repo_path(repo_path);
        let path2 = provider.shadow_repo_path(repo_path);

        // Should be deterministic for the same path
        assert_eq!(path1, path2);
    }
}
