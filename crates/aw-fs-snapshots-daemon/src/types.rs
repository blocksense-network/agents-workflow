use serde::{Deserialize, Serialize};

// TODO: Implement proper SSZ encoding/decoding
// For now using serde for compatibility
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub command: String,
    pub filesystem: Option<String>,
    pub snapshot: Option<String>,
    pub clone: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
    pub destination: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub mountpoint: Option<String>,
    pub path: Option<String>,
    pub error: Option<String>,
}

impl Request {
    pub fn ping() -> Self {
        Self {
            command: "ping".to_string(),
            filesystem: None,
            snapshot: None,
            clone: None,
            source: None,
            target: None,
            destination: None,
        }
    }

    pub fn clone_zfs(snapshot: String, clone: String) -> Self {
        Self {
            command: "clone".to_string(),
            filesystem: Some("zfs".to_string()),
            snapshot: Some(snapshot),
            clone: Some(clone),
            source: None,
            target: None,
            destination: None,
        }
    }

    pub fn snapshot_zfs(source: String, snapshot: String) -> Self {
        Self {
            command: "snapshot".to_string(),
            filesystem: Some("zfs".to_string()),
            snapshot: Some(snapshot),
            clone: None,
            source: Some(source),
            target: None,
            destination: None,
        }
    }

    pub fn delete_zfs(target: String) -> Self {
        Self {
            command: "delete".to_string(),
            filesystem: Some("zfs".to_string()),
            snapshot: None,
            clone: None,
            source: None,
            target: Some(target),
            destination: None,
        }
    }

    pub fn clone_btrfs(source: String, destination: String) -> Self {
        Self {
            command: "clone".to_string(),
            filesystem: Some("btrfs".to_string()),
            snapshot: None,
            clone: None,
            source: Some(source),
            target: None,
            destination: Some(destination),
        }
    }

    pub fn snapshot_btrfs(source: String, destination: String) -> Self {
        Self {
            command: "snapshot".to_string(),
            filesystem: Some("btrfs".to_string()),
            snapshot: None,
            clone: None,
            source: Some(source),
            target: None,
            destination: Some(destination),
        }
    }

    pub fn delete_btrfs(target: String) -> Self {
        Self {
            command: "delete".to_string(),
            filesystem: Some("btrfs".to_string()),
            snapshot: None,
            clone: None,
            source: None,
            target: Some(target),
            destination: None,
        }
    }
}

impl Response {
    pub fn success() -> Self {
        Self {
            success: true,
            mountpoint: None,
            path: None,
            error: None,
        }
    }

    pub fn success_with_mountpoint(mountpoint: String) -> Self {
        Self {
            success: true,
            mountpoint: Some(mountpoint),
            path: None,
            error: None,
        }
    }

    pub fn success_with_path(path: String) -> Self {
        Self {
            success: true,
            mountpoint: None,
            path: Some(path),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            mountpoint: None,
            path: None,
            error: Some(message),
        }
    }
}
