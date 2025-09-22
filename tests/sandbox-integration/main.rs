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
    };

    // Initialize components
    let sandbox = Sandbox::with_namespace_config(namespace_config)
        .with_process_config(process_config);

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

#[test]
fn test_filesystem_config_defaults() {
    let config = FilesystemConfig::default();

    // Verify default readonly paths include common system directories
    assert!(config.readonly_paths.contains(&"/etc".to_string()));
    assert!(config.readonly_paths.contains(&"/usr".to_string()));
    assert!(config.readonly_paths.contains(&"/bin".to_string()));
    assert!(config.bind_mounts.is_empty());
    assert!(config.working_dir.is_none());
}
