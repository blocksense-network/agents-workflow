//! Common utilities for FUSE integration tests

use agentfs_core::{FsConfig, FsCore};
// Note: AgentFsFuse is not directly imported here as it's only used in conditional compilation
use anyhow::{Context, Result};
use nix::unistd::Pid;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::{tempdir, TempDir};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Test configuration for FUSE integration tests
#[derive(Debug, Clone)]
pub struct FuseTestConfig {
    /// Mount point for the filesystem
    pub mount_point: PathBuf,
    /// Temporary directory for test files
    pub temp_dir: Arc<TempDir>,
    /// FUSE mount options
    pub mount_options: Vec<String>,
    /// Test timeout in seconds
    pub timeout_secs: u64,
    /// Filesystem configuration
    pub fs_config: FsConfig,
}

impl Default for FuseTestConfig {
    fn default() -> Self {
        let temp_dir = Arc::new(tempdir().expect("Failed to create temp dir"));
        let mount_point = temp_dir.path().join("mount");

        // Create mount point directory
        fs::create_dir_all(&mount_point).expect("Failed to create mount point");

        Self {
            mount_point,
            temp_dir,
            mount_options: vec![
                "--allow-other".to_string(),
                "--auto-unmount".to_string(),
            ],
            timeout_secs: 30,
            fs_config: FsConfig::default(),
        }
    }
}

/// Represents a mounted FUSE filesystem for testing
pub struct MountedFilesystem {
    /// Mount point
    pub mount_point: PathBuf,
    /// FUSE process handle
    pub process: Child,
    /// Test configuration
    pub config: FuseTestConfig,
}

impl MountedFilesystem {
    /// Mount an AgentFS filesystem for testing
    pub async fn mount(config: FuseTestConfig) -> Result<Self> {
        info!("Mounting AgentFS at {}", config.mount_point.display());

        // Start the FUSE process
        let fuse_binary = std::env::var("CARGO_BIN_EXE_agentfs-fuse-host")
            .unwrap_or_else(|_| "target/debug/agentfs-fuse-host".to_string());
        let mut cmd = Command::new(fuse_binary);
        cmd.arg(&config.mount_point)
            .arg("--allow-other")
            .arg("--auto-unmount");

        // Add any additional mount options
        for option in &config.mount_options {
            if option.starts_with("--") {
                cmd.arg(option);
            }
        }

        // Set up environment and I/O
        cmd.env("RUST_LOG", "debug")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Starting FUSE process: {:?}", cmd);
        let mut process = cmd.spawn()
            .context("Failed to start FUSE process")?;

        // Wait for mount to complete (check if mount point is accessible)
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(config.timeout_secs);

        while start_time.elapsed() < timeout_duration {
            if config.mount_point.exists() {
                // Try to list the directory to ensure it's mounted
                match fs::read_dir(&config.mount_point) {
                    Ok(_) => {
                        info!("Filesystem mounted successfully");
                        break;
                    }
                    Err(e) => {
                        debug!("Mount point exists but not ready: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        if start_time.elapsed() >= timeout_duration {
            // Cleanup on failure
            let _ = process.kill();
            let _ = process.wait();
            anyhow::bail!("Filesystem mount timed out after {}s", config.timeout_secs);
        }

        Ok(Self {
            mount_point: config.mount_point.clone(),
            process,
            config,
        })
    }

    /// Check if the filesystem is still mounted and responsive
    pub fn is_mounted(&self) -> bool {
        // Check if mount point exists and is a directory
        if !self.mount_point.exists() || !self.mount_point.is_dir() {
            return false;
        }

        // Try to list the directory
        match fs::read_dir(&self.mount_point) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get filesystem statistics
    pub fn get_stats(&self) -> Result<String> {
        let output = Command::new("df")
            .arg("-h")
            .arg(&self.mount_point)
            .output()
            .context("Failed to run df command")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Wait for filesystem to be unmounted and process to exit
    pub async fn wait_for_unmount(mut self) -> Result<()> {
        info!("Waiting for filesystem unmount");

        // For now, just wait synchronously - can add timeout later if needed
        match tokio::task::spawn_blocking(move || self.process.wait()).await {
            Ok(Ok(status)) => {
                if status.success() {
                    info!("Filesystem unmounted successfully");
                } else {
                    warn!("FUSE process exited with status: {:?}", status);
                }
                Ok(())
            }
            Ok(Err(e)) => {
                warn!("Failed to wait for FUSE process: {}", e);
                Err(e.into())
            }
            Err(e) => {
                warn!("Task join error: {}", e);
                Err(e.into())
            }
        }
    }
}

impl Drop for MountedFilesystem {
    fn drop(&mut self) {
        if let Ok(Some(_)) = self.process.try_wait() {
            // Process already exited, nothing to do
            return;
        }

        // Try to unmount gracefully first
        let _ = Command::new("fusermount3")
            .arg("-u")
            .arg(&self.mount_point)
            .status();

        // Force kill the process
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Test helper for creating test files and directories
pub struct TestFileSystem {
    /// Base directory for test files
    pub base_dir: PathBuf,
}

impl TestFileSystem {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Create a test file with content
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let full_path = self.base_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, content)?;
        Ok(())
    }

    /// Create a test directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let full_path = self.base_dir.join(path);
        fs::create_dir_all(full_path)?;
        Ok(())
    }

    /// Read a test file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let full_path = self.base_dir.join(path);
        Ok(fs::read_to_string(full_path)?)
    }

    /// Check if path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.base_dir.join(path).exists()
    }

    /// List directory contents
    pub fn list_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let full_path = self.base_dir.join(path);
        let entries = fs::read_dir(full_path)?
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        Ok(entries)
    }
}

/// Run pjdfstest compliance tests
pub async fn run_pjdfstest(mount_point: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        info!("Running pjdfstest compliance tests on {}", mount_point.display());

        let output = Command::new("pjdfstest")
            .arg(mount_point)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to run pjdfstest")?;

        if output.status.success() {
            info!("pjdfstest completed successfully");
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("pjdfstest failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
            Err(anyhow::anyhow!("pjdfstest failed with exit code {:?}", output.status.code()))
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        info!("pjdfstest not available on this platform, skipping");
        Ok(())
    }
}

/// Get current process ID
pub fn get_current_pid() -> u32 {
    std::process::id()
}

/// Wait for a file to exist with timeout
pub async fn wait_for_file<P: AsRef<Path>>(path: P, timeout_secs: u64) -> Result<()> {
    let path = path.as_ref();
    let timeout_duration = Duration::from_secs(timeout_secs);

    let start_time = Instant::now();
    while start_time.elapsed() < timeout_duration {
        if path.exists() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Err(anyhow::anyhow!("File {} did not appear within {}s", path.display(), timeout_secs))
}

/// Helper to create a temporary directory for testing
pub fn create_temp_test_dir(prefix: &str) -> Result<TempDir> {
    tempdir().with_context(|| format!("Failed to create temp dir with prefix: {}", prefix))
}

/// Measure execution time of a function
pub async fn measure_time<F, Fut, T>(operation: &str, f: F) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let start = Instant::now();
    info!("Starting operation: {}", operation);

    let result = f().await;

    let elapsed = start.elapsed();
    match &result {
        Ok(_) => info!("✅ {} completed in {:?}", operation, elapsed),
        Err(e) => warn!("❌ {} failed after {:?}: {}", operation, elapsed, e),
    }

    result
}
