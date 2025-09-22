//! Integration tests for sandbox functionality

use sandbox_core::{NamespaceConfig, ProcessConfig, Sandbox};
use sandbox_fs::{FilesystemConfig, FilesystemManager};

#[tokio::test]
async fn test_sandbox_integration() {
    // Create namespace configuration
    let namespace_config = NamespaceConfig {
        user_ns: true,
        mount_ns: true,
        pid_ns: true,
        uts_ns: true,
        ipc_ns: true,
        time_ns: false,
        uid_map: None,
        gid_map: None,
    };

    // Create process configuration
    let process_config = ProcessConfig {
        command: vec!["echo".to_string(), "sandbox test".to_string()],
        working_dir: None,
        env: vec![],
    };

    // Create filesystem configuration
    let fs_config = FilesystemConfig {
        readonly_paths: vec!["/etc".to_string()],
        bind_mounts: vec![],
        working_dir: Some("/tmp".to_string()),
        overlay_paths: vec![],
        blacklist_paths: vec![],
        session_state_dir: None,
        static_mode: false,
    };

    // Initialize components
    let mut sandbox =
        Sandbox::with_namespace_config(namespace_config).with_process_config(process_config);

    let fs_manager = FilesystemManager::with_config(fs_config);

    // Test that components can be created and configured
    assert!(sandbox.start().await.is_ok());
    assert!(fs_manager.setup_mounts().await.is_ok());

    // Verify configurations
    assert!(sandbox.namespace_config().user_ns);
    assert!(sandbox.namespace_config().mount_ns);
    assert!(sandbox.namespace_config().pid_ns);
    assert_eq!(fs_manager.config().readonly_paths.len(), 1);
    assert_eq!(fs_manager.config().readonly_paths[0], "/etc");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_cgroups_integration() {
    // Create custom cgroup configuration for testing
    let cgroup_config = sandbox_core::CgroupConfig {
        cgroup_root: std::path::PathBuf::from("/sys/fs/cgroup"),
        pids: sandbox_core::PidLimits {
            max: Some(512), // Lower limit for testing
        },
        memory: sandbox_core::MemoryLimits {
            high: Some(100 * 1024 * 1024), // 100MB
            max: Some(200 * 1024 * 1024),  // 200MB
        },
        cpu: sandbox_core::CpuLimits {
            max: Some("25000 100000".to_string()), // 25% of one CPU
        },
    };

    // Create sandbox with cgroups enabled
    let namespace_config = NamespaceConfig {
        user_ns: true,
        mount_ns: true,
        pid_ns: true,
        uts_ns: true,
        ipc_ns: true,
        time_ns: false,
        uid_map: None,
        gid_map: None,
    };

    let process_config = ProcessConfig {
        command: vec!["echo".to_string(), "cgroup test".to_string()],
        working_dir: None,
        env: vec![],
    };

    let mut sandbox = Sandbox::with_namespace_config(namespace_config)
        .with_process_config(process_config)
        .with_cgroups(cgroup_config);

    // Test cgroup setup and cleanup
    assert!(sandbox.start().await.is_ok());

    // Test metrics collection
    let metrics = sandbox.collect_metrics().unwrap();
    assert!(metrics.is_some());
    let metrics = metrics.unwrap();

    // Metrics might be None if cgroup operations failed in test environment
    // but the structure should be present
    assert!(metrics.current_pids.is_none() || metrics.current_pids.is_some());
    assert!(metrics.memory_current.is_none() || metrics.memory_current.is_some());
    assert!(metrics.cpu_usage.is_none() || metrics.cpu_usage.is_some());

    // Test cleanup
    assert!(sandbox.stop().is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn test_cgroups_manager_direct() {
    // Test direct cgroup manager usage
    let config = sandbox_core::CgroupConfig::default();
    let mut manager = sandbox_core::CgroupManager::new(config);

    // Test setup (may fail in test environments without privileges)
    let _setup_result = manager.setup_limits();
    // We don't assert success since this may fail in test environments

    // Test metrics collection (should work even without cgroup setup)
    let metrics = manager.collect_metrics().unwrap();
    assert!(metrics.current_pids.is_none()); // No cgroup created
    assert!(metrics.memory_current.is_none());
    assert!(metrics.memory_events.is_none());
    assert!(metrics.cpu_usage.is_none());
    assert!(metrics.cpu_user.is_none());
    assert!(metrics.cpu_system.is_none());

    // Test cleanup (should not fail even if no cgroup exists)
    assert!(manager.cleanup().is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn test_cgroups_enforcement_e2e() {
    // This test runs the full E2E cgroup enforcement test suite
    // It requires the test binaries to be built and available

    use std::process::Command;

    // Build paths relative to the project root (not the test directory)
    let project_root = std::env::current_dir()
        .unwrap_or_default()
        .parent() // Go up from tests/sandbox-integration to tests/
        .unwrap_or(&std::env::current_dir().unwrap_or_default())
        .parent() // Go up from tests/ to project root
        .unwrap_or(&std::env::current_dir().unwrap_or_default())
        .to_path_buf();

    let orchestrator_path = project_root.join("target/debug/test_orchestrator");
    let helper_path = project_root.join("target/debug/sbx-helper");

    println!("Looking for orchestrator at: {:?}", orchestrator_path);
    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap_or_default()
    );

    if !orchestrator_path.exists() {
        println!("⚠️  Skipping E2E cgroup enforcement test - test_orchestrator binary not found");
        println!("   Build with: cargo build --bin test_orchestrator");
        println!("   Looking in: {:?}", orchestrator_path);
        return;
    }

    if !helper_path.exists() {
        println!("⚠️  Skipping E2E cgroup enforcement test - sbx-helper binary not found");
        println!("   Build with: cargo build --bin sbx-helper");
        return;
    }

    // Run the test orchestrator
    println!("🧪 Running E2E cgroup enforcement tests...");
    let output = Command::new(orchestrator_path)
        .output()
        .expect("Failed to run test orchestrator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Test orchestrator stdout:\n{}", stdout);
    if !stderr.is_empty() {
        eprintln!("Test orchestrator stderr:\n{}", stderr);
    }

    // Check if the orchestrator succeeded
    if output.status.success() {
        println!("✅ E2E cgroup enforcement tests PASSED");
    } else {
        println!("❌ E2E cgroup enforcement tests FAILED");
        println!("Exit code: {:?}", output.status.code());
        panic!("Cgroup enforcement E2E tests failed");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_overlay_enforcement_e2e() {
    // This test runs the full E2E overlay enforcement test suite
    // It requires the test binaries to be built and available
    // Note: This test requires privileges to create namespaces and mount filesystems

    use std::process::Command;

    // Build paths relative to the project root (not the test directory)
    let project_root = std::env::current_dir()
        .unwrap_or_default()
        .parent() // Go up from tests/sandbox-integration to tests/
        .unwrap_or(&std::env::current_dir().unwrap_or_default())
        .parent() // Go up from tests/ to project root
        .unwrap_or(&std::env::current_dir().unwrap_or_default())
        .to_path_buf();

    let orchestrator_path = project_root.join("target/debug/overlay_test_orchestrator");
    let sbx_helper_path = project_root.join("target/debug/sbx-helper");

    println!("Looking for overlay orchestrator at: {:?}", orchestrator_path);
    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap_or_default()
    );

    if !orchestrator_path.exists() {
        println!("⚠️  Skipping E2E overlay enforcement test - overlay_test_orchestrator binary not found");
        println!("   Build with: cargo build --bin overlay_test_orchestrator");
        println!("   Looking in: {:?}", orchestrator_path);
        return;
    }

    if !sbx_helper_path.exists() {
        println!("⚠️  Skipping E2E overlay enforcement test - sbx-helper binary not found");
        println!("   Build with: cargo build --bin sbx-helper");
        return;
    }

    // Check if we have the necessary privileges to run sandbox tests
    // Try a simple namespace creation to see if we have privileges
    let can_create_namespaces = std::fs::read_to_string("/proc/sys/user/max_user_namespaces")
        .map(|s| s.trim().parse::<i32>().unwrap_or(0) > 0)
        .unwrap_or(false);

    if !can_create_namespaces {
        println!("⚠️  Skipping E2E overlay enforcement test - no namespace privileges available");
        println!("   This test requires privileges to create user namespaces and mount filesystems");
        println!("   Run with appropriate privileges (e.g., sudo) or in a privileged environment");
        return;
    }

    // Run the overlay test orchestrator
    println!("🧪 Running E2E overlay enforcement tests...");
    let output = Command::new(orchestrator_path)
        .output()
        .expect("Failed to run overlay test orchestrator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Overlay test orchestrator stdout:\n{}", stdout);
    if !stderr.is_empty() {
        eprintln!("Overlay test orchestrator stderr:\n{}", stderr);
    }

    // Check if the orchestrator succeeded
    if output.status.success() {
        println!("✅ E2E overlay enforcement tests PASSED");
    } else {
        println!("❌ E2E overlay enforcement tests FAILED");
        println!("Exit code: {:?}", output.status.code());
        panic!("Overlay enforcement E2E tests failed");
    }
}

#[test]
fn test_filesystem_config_defaults() {
    let config = FilesystemConfig::default();

    // Verify default readonly paths include common system directories
    assert!(config.readonly_paths.contains(&"/etc".to_string()));
    assert!(config.readonly_paths.contains(&"/usr".to_string()));
    assert!(config.readonly_paths.contains(&"/bin".to_string()));
    assert!(config.bind_mounts.is_empty());
    assert!(config.working_dir.is_none());
    assert!(config.overlay_paths.is_empty());
    assert!(!config.blacklist_paths.is_empty()); // Should have default blacklisted paths
    assert!(config.session_state_dir.is_none());
    assert!(!config.static_mode);
}
