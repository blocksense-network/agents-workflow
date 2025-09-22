//! XPC control plane implementation for FSKit adapter

use super::FsKitAdapter;
use agentfs_proto::*;
use serde_json;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use tokio::sync::mpsc;

/// XPC service for handling control operations
#[cfg(target_os = "macos")]
pub struct XpcControlService {
    adapter: Arc<FsKitAdapter>,
    service_name: String,
}

#[cfg(target_os = "macos")]
impl XpcControlService {
    pub fn new(adapter: Arc<FsKitAdapter>, service_name: String) -> Self {
        Self {
            adapter,
            service_name,
        }
    }

    /// Start the XPC service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting XPC control service: {}", self.service_name);

        // In a real implementation, this would set up XPC listeners
        // For now, this is a simplified async service
        let (_tx, mut _rx) = mpsc::channel::<String>(32);

        // Simulate XPC message handling
        // tokio::spawn(async move {
        //     while let Some(request) = rx.recv().await {
        //         let response = self.handle_request(&request).await;
        //         // Send response back via XPC
        //         println!("XPC Response: {:?}", response);
        //     }
        // });

        Ok(())
    }

    /// Handle incoming XPC request
    async fn handle_request(&self, request_json: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse the request
        let envelope: MessageEnvelope<serde_json::Value> = serde_json::from_str(request_json)?;

        if envelope.version != "1" {
            let error = ErrorResponse {
                error: format!("Unsupported version: {}", envelope.version),
                code: Some(95), // ENOTSUP
            };
            return Ok(serde_json::to_string(&error)?);
        }

        // Route based on operation
        match envelope.payload.get("op").and_then(|v| v.as_str()) {
            Some("snapshot.create") => {
                let request: SnapshotCreateRequest = serde_json::from_value(envelope.payload)?;
                self.handle_snapshot_create(request).await
            }
            Some("snapshot.list") => {
                let request: SnapshotListRequest = serde_json::from_value(envelope.payload)?;
                self.handle_snapshot_list(request).await
            }
            Some("branch.create") => {
                let request: BranchCreateRequest = serde_json::from_value(envelope.payload)?;
                self.handle_branch_create(request).await
            }
            Some("branch.bind") => {
                let request: BranchBindRequest = serde_json::from_value(envelope.payload)?;
                self.handle_branch_bind(request).await
            }
            _ => {
                let error = ErrorResponse {
                    error: "Unknown operation".to_string(),
                    code: Some(22), // EINVAL
                };
                Ok(serde_json::to_string(&error)?)
            }
        }
    }

    async fn handle_snapshot_create(&self, request: SnapshotCreateRequest) -> Result<String, Box<dyn std::error::Error>> {
        match self.adapter.core().snapshot_create(request.name.as_deref()) {
            Ok(snapshot_id) => {
                let response = SnapshotCreateResponse { snapshot_id };
                Ok(serde_json::to_string(&response)?)
            }
            Err(e) => {
                let error = ErrorResponse {
                    error: e.to_string(),
                    code: Some(self.map_error_code(&e)),
                };
                Ok(serde_json::to_string(&error)?)
            }
        }
    }

    async fn handle_snapshot_list(&self, _request: SnapshotListRequest) -> Result<String, Box<dyn std::error::Error>> {
        let snapshots = self.adapter.core().snapshot_list();
        let snapshot_infos: Vec<SnapshotInfo> = snapshots
            .into_iter()
            .map(|(id, name)| SnapshotInfo { id, name })
            .collect();

        let response = SnapshotListResponse {
            snapshots: snapshot_infos,
        };
        Ok(serde_json::to_string(&response)?)
    }

    async fn handle_branch_create(&self, request: BranchCreateRequest) -> Result<String, Box<dyn std::error::Error>> {
        match self.adapter.core().branch_create_from_snapshot(request.from_snapshot, request.name.as_deref()) {
            Ok(branch_id) => {
                let response = BranchCreateResponse { branch_id };
                Ok(serde_json::to_string(&response)?)
            }
            Err(e) => {
                let error = ErrorResponse {
                    error: e.to_string(),
                    code: Some(self.map_error_code(&e)),
                };
                Ok(serde_json::to_string(&error)?)
            }
        }
    }

    async fn handle_branch_bind(&self, request: BranchBindRequest) -> Result<String, Box<dyn std::error::Error>> {
        let pid = request.pid.unwrap_or(std::process::id());

        // Note: bind_process_to_branch_with_pid doesn't exist in the current API
        // We'll need to extend the core API for this
        match self.adapter.core().bind_process_to_branch(request.branch_id) {
            Ok(()) => {
                let response = BranchBindResponse {};
                Ok(serde_json::to_string(&response)?)
            }
            Err(e) => {
                let error = ErrorResponse {
                    error: e.to_string(),
                    code: Some(self.map_error_code(&e)),
                };
                Ok(serde_json::to_string(&error)?)
            }
        }
    }

    fn map_error_code(&self, error: &agentfs_core::FsError) -> i32 {
        match error {
            agentfs_core::FsError::NotFound => 2,      // ENOENT
            agentfs_core::FsError::AlreadyExists => 17, // EEXIST
            agentfs_core::FsError::AccessDenied => 13,  // EACCES
            agentfs_core::FsError::InvalidArgument => 22, // EINVAL
            agentfs_core::FsError::Busy => 16,          // EBUSY
            agentfs_core::FsError::NoSpace => 28,       // ENOSPC
            agentfs_core::FsError::Unsupported => 95,   // ENOTSUP
            _ => 5, // EIO
        }
    }
}

/// Stub XPC implementation for non-macOS
#[cfg(not(target_os = "macos"))]
pub struct XpcControlService;

#[cfg(not(target_os = "macos"))]
impl XpcControlService {
    pub fn new(_adapter: Arc<FsKitAdapter>, _service_name: String) -> Self {
        Self
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Err("XPC is only available on macOS".into())
    }
}
