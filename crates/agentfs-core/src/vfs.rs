//! Virtual filesystem implementation for AgentFS Core

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    Attributes, BranchId, BranchInfo, ContentId, DirEntry, EventKind, EventSink, FileMode, FileTimes, FsConfig, FsStats, HandleId, LockKind, LockRange, OpenOptions, ShareMode, SnapshotId, StreamSpec, SubscriptionId,
};
use crate::error::{FsError, FsResult};
use crate::storage::StorageBackend;

/// Internal node ID for filesystem nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(u64);

/// Filesystem node types
#[derive(Clone, Debug)]
pub(crate) enum NodeKind {
    File {
        streams: HashMap<String, (ContentId, u64)>, // stream_name -> (content_id, size)
    },
    Directory { children: HashMap<String, NodeId> },
    Symlink { target: String },
}

/// Filesystem node
#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub times: FileTimes,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub xattrs: HashMap<String, Vec<u8>>, // Extended attributes
}

/// Open file handle
#[derive(Debug)]
pub(crate) struct Handle {
    pub id: HandleId,
    pub node_id: NodeId,
    pub position: u64,
    pub options: OpenOptions,
    pub deleted: bool, // For delete-on-close semantics
}

/// Snapshot containing immutable tree state
#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub id: SnapshotId,
    pub root_id: NodeId,
    pub name: Option<String>,
}

/// Branch state containing a tree of nodes (writable clone of a snapshot)
#[derive(Clone, Debug)]
pub(crate) struct Branch {
    pub id: BranchId,
    pub root_id: NodeId,
    pub parent_snapshot: Option<SnapshotId>,
    pub name: Option<String>,
}

/// Active byte-range lock
#[derive(Clone, Debug)]
pub(crate) struct ActiveLock {
    pub handle_id: HandleId,
    pub range: LockRange,
}

/// Lock manager for tracking byte-range locks per node
#[derive(Clone, Debug)]
pub(crate) struct LockManager {
    pub locks: HashMap<NodeId, Vec<ActiveLock>>,
}

/// The main filesystem core implementation
pub struct FsCore {
    config: FsConfig,
    storage: Arc<dyn StorageBackend>,
    nodes: Mutex<HashMap<NodeId, Node>>,
    pub(crate) snapshots: Mutex<HashMap<SnapshotId, Snapshot>>,
    pub(crate) branches: Mutex<HashMap<BranchId, Branch>>,
    handles: Mutex<HashMap<HandleId, Handle>>,
    next_node_id: Mutex<u64>,
    next_handle_id: Mutex<u64>,
    next_subscription_id: Mutex<u64>,
    pub(crate) process_branches: Mutex<HashMap<u32, BranchId>>, // Process ID -> Branch ID mapping
    locks: Mutex<LockManager>, // Byte-range lock manager
    event_subscriptions: Mutex<HashMap<SubscriptionId, Arc<dyn EventSink>>>,
}

impl FsCore {
    pub fn new(config: FsConfig) -> FsResult<Self> {
        let storage: Arc<dyn StorageBackend> = Arc::new(crate::storage::InMemoryBackend::new());

        let mut core = Self {
            config,
            storage,
            nodes: Mutex::new(HashMap::new()),
            snapshots: Mutex::new(HashMap::new()),
            branches: Mutex::new(HashMap::new()),
            handles: Mutex::new(HashMap::new()),
            next_node_id: Mutex::new(1),
            next_handle_id: Mutex::new(1),
            next_subscription_id: Mutex::new(1),
            process_branches: Mutex::new(HashMap::new()), // No processes initially bound
            locks: Mutex::new(LockManager {
                locks: HashMap::new(),
            }),
            event_subscriptions: Mutex::new(HashMap::new()),
        };

        // Create root directory
        core.create_root_directory()?;
        Ok(core)
    }

    fn create_root_directory(&mut self) -> FsResult<()> {
        let root_node_id = self.allocate_node_id();
        let now = Self::current_timestamp();

        let root_node = Node {
            id: root_node_id,
            kind: NodeKind::Directory {
                children: HashMap::new(),
            },
            times: FileTimes {
                atime: now,
                mtime: now,
                ctime: now,
                birthtime: now,
            },
            mode: 0o755,
            uid: self.config.security.default_uid,
            gid: self.config.security.default_gid,
            xattrs: HashMap::new(),
        };

        let default_branch = Branch {
            id: BranchId::DEFAULT,
            root_id: root_node_id,
            parent_snapshot: None, // Default branch has no parent snapshot
            name: Some("default".to_string()),
        };

        self.nodes.lock().unwrap().insert(root_node_id, root_node);
        self.branches.lock().unwrap().insert(default_branch.id, default_branch);

        Ok(())
    }

    fn allocate_node_id(&self) -> NodeId {
        let mut next_id = self.next_node_id.lock().unwrap();
        let id = NodeId(*next_id);
        *next_id += 1;
        id
    }

    fn allocate_handle_id(&self) -> HandleId {
        let mut next_id = self.next_handle_id.lock().unwrap();
        let id = HandleId::new(*next_id);
        *next_id += 1;
        id
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn current_process_id() -> u32 {
        std::process::id()
    }

    fn current_branch_for_process(&self) -> BranchId {
        let pid = Self::current_process_id();
        let process_branches = self.process_branches.lock().unwrap();
        *process_branches.get(&pid).unwrap_or(&BranchId::DEFAULT)
    }

    /// Resolve a path to a node ID and parent information (read-only)
    fn resolve_path(&self, path: &Path) -> FsResult<(NodeId, Option<(NodeId, String)>)> {
        let current_branch = self.current_branch_for_process();
        let branches = self.branches.lock().unwrap();
        let branch = branches.get(&current_branch).ok_or(FsError::NotFound)?;
        let mut current_node_id = branch.root_id;

        let components: Vec<&str> = path
            .components()
            .filter_map(|c| match c {
                std::path::Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .collect();

        if components.is_empty() {
            // Root directory
            return Ok((current_node_id, None));
        }

        let nodes = self.nodes.lock().unwrap();
        let mut parent_node_id = None;
        let mut parent_name = None;

        for (i, component) in components.iter().enumerate() {
            let node = nodes.get(&current_node_id).ok_or(FsError::NotFound)?;

            match &node.kind {
                NodeKind::Directory { children } => {
                    if let Some(child_id) = children.get(*component) {
                        if i == components.len() - 1 {
                            // Last component
                            return Ok((*child_id, Some((current_node_id, component.to_string()))));
                        } else {
                            parent_node_id = Some(current_node_id);
                            parent_name = Some(component.to_string());
                            current_node_id = *child_id;
                        }
                    } else {
                        return Err(FsError::NotFound);
                    }
                }
                NodeKind::File { .. } => {
                    if i == components.len() - 1 {
                        // Last component is a file
                        return Ok((current_node_id, Some((current_node_id, component.to_string()))));
                    } else {
                        return Err(FsError::NotADirectory);
                    }
                }
                NodeKind::Symlink { .. } => {
                    if i == components.len() - 1 {
                        // Last component is a symlink
                        return Ok((current_node_id, Some((current_node_id, component.to_string()))));
                    } else {
                        return Err(FsError::NotADirectory);
                    }
                }
            }
        }

        Ok((current_node_id, parent_node_id.zip(parent_name)))
    }


    /// Create a new file node
    fn create_file_node(&self, content_id: ContentId) -> FsResult<NodeId> {
        let node_id = self.allocate_node_id();
        let now = Self::current_timestamp();

        let mut streams = HashMap::new();
        streams.insert("".to_string(), (content_id, 0)); // Default unnamed stream

        let node = Node {
            id: node_id,
            kind: NodeKind::File { streams },
            times: FileTimes {
                atime: now,
                mtime: now,
                ctime: now,
                birthtime: now,
            },
            mode: 0o644,
            uid: self.config.security.default_uid,
            gid: self.config.security.default_gid,
            xattrs: HashMap::new(),
        };

        self.nodes.lock().unwrap().insert(node_id, node);
        Ok(node_id)
    }

    /// Create a new directory node
    fn create_directory_node(&self) -> FsResult<NodeId> {
        let node_id = self.allocate_node_id();
        let now = Self::current_timestamp();

        let node = Node {
            id: node_id,
            kind: NodeKind::Directory {
                children: HashMap::new(),
            },
            times: FileTimes {
                atime: now,
                mtime: now,
                ctime: now,
                birthtime: now,
            },
            mode: 0o755,
            uid: self.config.security.default_uid,
            gid: self.config.security.default_gid,
            xattrs: HashMap::new(),
        };

        self.nodes.lock().unwrap().insert(node_id, node);
        Ok(node_id)
    }

    /// Change ownership of a node addressed by path
    pub fn set_owner(&self, path: &Path, uid: u32, gid: u32) -> FsResult<()> {
        let (node_id, _) = self.resolve_path(path)?;
        let mut nodes = self.nodes.lock().unwrap();
        let node = nodes.get_mut(&node_id).ok_or(FsError::NotFound)?;
        node.uid = uid;
        node.gid = gid;
        node.times.ctime = Self::current_timestamp();
        Ok(())
    }

    /// Percent-encode arbitrary bytes to a safe internal string name
    fn percent_encode_name(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 3);
        for &b in bytes {
            let is_safe = (b'A'..=b'Z').contains(&b)
                || (b'a'..=b'z').contains(&b)
                || (b'0'..=b'9').contains(&b)
                || matches!(b, b'-' | b'_' | b'.');
            if is_safe {
                s.push(b as char);
            } else {
                s.push('%');
                s.push_str(&format!("{:02X}", b));
            }
        }
        s
    }

    /// Create a child under a parent directory by parent node id and raw name bytes.
    /// Returns created node id.
    pub fn create_child_by_id(
        &self,
        parent_id_u64: u64,
        name_bytes: &[u8],
        item_type: u32,
        mode: u32,
    ) -> FsResult<u64> {
        let parent_id = NodeId(parent_id_u64);
        let mut nodes = self.nodes.lock().unwrap();
        let parent_node = nodes.get_mut(&parent_id).ok_or(FsError::NotFound)?;

        // Determine internal name used for map lookup
        let internal_name = match std::str::from_utf8(name_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => Self::percent_encode_name(name_bytes),
        };

        // Ensure parent is a directory and the child doesn't exist
        match &mut parent_node.kind {
            NodeKind::Directory { children } => {
                if children.contains_key(&internal_name) {
                    return Err(FsError::AlreadyExists);
                }
            }
            _ => return Err(FsError::NotADirectory),
        }
        drop(nodes);

        // Create the node
        let new_node_id = match item_type {
            0 => {
                // file
                let content_id = self.storage.allocate(&[])?;
                self.create_file_node(content_id)?
            }
            1 => {
                // directory
                self.create_directory_node()?
            }
            _ => return Err(FsError::InvalidArgument),
        };

        // Apply mode
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(n) = nodes.get_mut(&new_node_id) {
                n.mode = mode;
                // Preserve original raw name in xattr for later round-trip
                n.xattrs.insert(
                    "user.agentfs.rawname".to_string(),
                    name_bytes.to_vec(),
                );
            }
        }

        // Insert into parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent.kind {
                    children.insert(internal_name, new_node_id);
                }
            }
        }

        Ok(new_node_id.0)
    }

    /// Get attributes of a child under a parent directory by raw name bytes
    pub fn getattr_child_by_id_name(&self, parent_id_u64: u64, name_bytes: &[u8]) -> FsResult<Attributes> {
        let parent_id = NodeId(parent_id_u64);
        let internal_name = match std::str::from_utf8(name_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => Self::percent_encode_name(name_bytes),
        };

        let nodes = self.nodes.lock().unwrap();
        let parent = nodes.get(&parent_id).ok_or(FsError::NotFound)?;
        let child_id = match &parent.kind {
            NodeKind::Directory { children } => children.get(&internal_name).ok_or(FsError::NotFound).copied()?,
            _ => return Err(FsError::NotADirectory),
        };
        drop(nodes);
        self.get_node_attributes(child_id)
    }

    /// Resolve child node id by parent id and raw name bytes
    pub fn resolve_child_id_by_id_name(&self, parent_id_u64: u64, name_bytes: &[u8]) -> FsResult<u64> {
        let parent_id = NodeId(parent_id_u64);
        let internal_name = match std::str::from_utf8(name_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => Self::percent_encode_name(name_bytes),
        };

        let nodes = self.nodes.lock().unwrap();
        let parent = nodes.get(&parent_id).ok_or(FsError::NotFound)?;
        let child_id = match &parent.kind {
            NodeKind::Directory { children } => children.get(&internal_name).ok_or(FsError::NotFound).copied()?,
            _ => return Err(FsError::NotADirectory),
        };
        Ok(child_id.0)
    }

    /// Clone a node for copy-on-write (creates a new node with the same content)
    fn clone_node_cow(&self, node_id: NodeId) -> FsResult<NodeId> {
        self.clone_node_cow_recursive(node_id)
    }

    /// Recursively clone a node and all its children for copy-on-write
    fn clone_node_cow_recursive(&self, node_id: NodeId) -> FsResult<NodeId> {
        // First, get the node data
        let node = {
            let nodes = self.nodes.lock().unwrap();
            nodes.get(&node_id).ok_or(FsError::NotFound)?.clone()
        };

        let new_node_id = self.allocate_node_id();
        let mut new_node = node.clone();
        // xattrs are already cloned by the derive(Clone) on Node

        // For files, we need to clone all streams in storage
        if let NodeKind::File { streams } = &new_node.kind {
            let mut new_streams = HashMap::new();
            for (stream_name, (content_id, size)) in streams {
                let new_content_id = self.storage.clone_cow(*content_id)?;
                new_streams.insert(stream_name.clone(), (new_content_id, *size));
            }
            new_node.kind = NodeKind::File {
                streams: new_streams,
            };
        }
        // For directories, we recursively clone all children
        else if let NodeKind::Directory { children } = &new_node.kind {
            let mut new_children = HashMap::new();
            for (name, child_id) in children {
                let new_child_id = self.clone_node_cow_recursive(*child_id)?;
                new_children.insert(name.clone(), new_child_id);
            }
            new_node.kind = NodeKind::Directory {
                children: new_children,
            };
        }

        // Insert the new node
        {
            let mut nodes = self.nodes.lock().unwrap();
            nodes.insert(new_node_id, new_node);
        }
        Ok(new_node_id)
    }

    /// Clone a branch's root directory for copy-on-write
    fn clone_branch_root_cow(&self, branch_id: BranchId) -> FsResult<()> {
        let mut branches = self.branches.lock().unwrap();
        let branch = branches.get_mut(&branch_id).ok_or(FsError::NotFound)?;

        // Only clone if the branch shares its root with a snapshot
        if let Some(snapshot_id) = branch.parent_snapshot {
            let snapshots = self.snapshots.lock().unwrap();
            if let Some(snapshot) = snapshots.get(&snapshot_id) {
                if branch.root_id == snapshot.root_id {
                    // Clone the root directory
                    let new_root_id = self.clone_node_cow(branch.root_id)?;
                    branch.root_id = new_root_id;
                }
            }
        }

        Ok(())
    }

    /// Update node timestamps
    fn update_node_times(&self, node_id: NodeId, times: FileTimes) {
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(node) = nodes.get_mut(&node_id) {
            node.times = times;
        }
    }

    /// Get node attributes
    fn get_node_attributes(&self, node_id: NodeId) -> FsResult<Attributes> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        let (len, is_dir, is_symlink) = match &node.kind {
            NodeKind::File { streams } => {
                // Size is the size of the unnamed stream (default data stream)
                let size = streams.get("").map(|(_, size)| *size).unwrap_or(0);
                (size, false, false)
            }
            NodeKind::Directory { .. } => (0, true, false),
            NodeKind::Symlink { target } => (target.len() as u64, false, true),
        };

        Ok(Attributes {
            len,
            times: node.times,
            uid: node.uid,
            gid: node.gid,
            is_dir,
            is_symlink,
            mode_user: FileMode {
                read: true,
                write: true,
                exec: false,
            },
            mode_group: FileMode {
                read: true,
                write: false,
                exec: false,
            },
            mode_other: FileMode {
                read: true,
                write: false,
                exec: false,
            },
        })
    }

    // Snapshot operations
    pub fn snapshot_create(&self, name: Option<&str>) -> FsResult<SnapshotId> {
        let current_branch = self.current_branch_for_process();
        let branches = self.branches.lock().unwrap();
        let branch = branches.get(&current_branch).ok_or(FsError::NotFound)?;

        let snapshot_id = SnapshotId::new();
        let snapshot = Snapshot {
            id: snapshot_id,
            root_id: branch.root_id,
            name: name.map(|s| s.to_string()),
        };

        self.snapshots.lock().unwrap().insert(snapshot_id, snapshot);

        // Emit event
        self.emit_event(EventKind::SnapshotCreated {
            id: snapshot_id,
            name: name.map(|s| s.to_string()),
        });

        Ok(snapshot_id)
    }

    pub fn snapshot_list(&self) -> Vec<(SnapshotId, Option<String>)> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.values()
            .map(|s| (s.id, s.name.clone()))
            .collect()
    }

    pub fn snapshot_delete(&self, snapshot_id: SnapshotId) -> FsResult<()> {
        let mut snapshots = self.snapshots.lock().unwrap();
        let branches = self.branches.lock().unwrap();

        // Check if any branches depend on this snapshot
        let has_dependents = branches.values()
            .any(|b| b.parent_snapshot == Some(snapshot_id));

        if has_dependents {
            return Err(FsError::Busy); // Cannot delete snapshot with dependent branches
        }

        snapshots.remove(&snapshot_id);
        Ok(())
    }

    // Branch operations
    pub fn branch_create_from_snapshot(&self, snapshot_id: SnapshotId, name: Option<&str>) -> FsResult<BranchId> {
        let snapshots = self.snapshots.lock().unwrap();
        let snapshot = snapshots.get(&snapshot_id).ok_or(FsError::NotFound)?;

        // Clone the snapshot's root directory for the branch (immediate CoW for directory structure)
        let branch_root_id = self.clone_node_cow(snapshot.root_id)?;

        let branch_id = BranchId::new();
        let branch = Branch {
            id: branch_id,
            root_id: branch_root_id, // Branch gets its own copy of the directory structure
            parent_snapshot: Some(snapshot_id),
            name: name.map(|s| s.to_string()),
        };

        self.branches.lock().unwrap().insert(branch_id, branch);

        // Emit event
        self.emit_event(EventKind::BranchCreated {
            id: branch_id,
            name: name.map(|s| s.to_string()),
        });

        Ok(branch_id)
    }

    pub fn branch_create_from_current(&self, name: Option<&str>) -> FsResult<BranchId> {
        let current_branch = self.current_branch_for_process();
        let branches = self.branches.lock().unwrap();
        let branch = branches.get(&current_branch).ok_or(FsError::NotFound)?;

        // Clone the current branch's root directory for the new branch
        let new_branch_root_id = self.clone_node_cow(branch.root_id)?;

        let branch_id = BranchId::new();
        let new_branch = Branch {
            id: branch_id,
            root_id: new_branch_root_id, // New branch gets its own copy of the directory structure
            parent_snapshot: None, // Not based on a snapshot
            name: name.map(|s| s.to_string()),
        };

        drop(branches);
        self.branches.lock().unwrap().insert(branch_id, new_branch);
        Ok(branch_id)
    }

    pub fn branch_list(&self) -> Vec<BranchInfo> {
        let branches = self.branches.lock().unwrap();
        branches.values()
            .map(|b| BranchInfo {
                id: b.id,
                parent: b.parent_snapshot,
                name: b.name.clone(),
            })
            .collect()
    }

    // Process binding operations
    pub fn bind_process_to_branch(&self, branch_id: BranchId) -> FsResult<()> {
        self.bind_process_to_branch_with_pid(branch_id, Self::current_process_id())
    }

    pub fn bind_process_to_branch_with_pid(&self, branch_id: BranchId, pid: u32) -> FsResult<()> {
        let branches = self.branches.lock().unwrap();
        if !branches.contains_key(&branch_id) {
            return Err(FsError::NotFound);
        }
        drop(branches);

        let mut process_branches = self.process_branches.lock().unwrap();
        process_branches.insert(pid, branch_id);
        Ok(())
    }

    pub fn unbind_process(&self) -> FsResult<()> {
        self.unbind_process_with_pid(Self::current_process_id())
    }

    pub fn unbind_process_with_pid(&self, pid: u32) -> FsResult<()> {
        let mut process_branches = self.process_branches.lock().unwrap();
        process_branches.remove(&pid);
        Ok(())
    }

    // Event subscription operations
    pub fn subscribe_events(&self, cb: Arc<dyn EventSink>) -> FsResult<SubscriptionId> {
        let mut subscriptions = self.event_subscriptions.lock().unwrap();
        let mut next_id = self.next_subscription_id.lock().unwrap();
        let subscription_id = SubscriptionId::new(*next_id);
        *next_id += 1;
        subscriptions.insert(subscription_id, cb);
        Ok(subscription_id)
    }

    pub fn unsubscribe_events(&self, sub: SubscriptionId) -> FsResult<()> {
        let mut subscriptions = self.event_subscriptions.lock().unwrap();
        if subscriptions.remove(&sub).is_none() {
            return Err(FsError::NotFound);
        }
        Ok(())
    }

    // Statistics
    pub fn stats(&self) -> FsStats {
        let branches = self.branches.lock().unwrap();
        let snapshots = self.snapshots.lock().unwrap();
        let handles = self.handles.lock().unwrap();

        // For now, we only track in-memory storage
        // TODO: Add actual byte counting when storage backend supports it
        let bytes_in_memory = 0; // Placeholder
        let bytes_spilled = 0; // Placeholder

        FsStats {
            branches: branches.len() as u32,
            snapshots: snapshots.len() as u32,
            open_handles: handles.len() as u32,
            bytes_in_memory,
            bytes_spilled,
        }
    }

    // Helper method to emit events to all subscribers
    fn emit_event(&self, event: EventKind) {
        if !self.config.track_events {
            return;
        }

        let subscriptions = self.event_subscriptions.lock().unwrap();
        for sink in subscriptions.values() {
            sink.on_event(&event);
        }
    }

    // File operations
    pub fn create(&self, path: &Path, opts: &OpenOptions) -> FsResult<HandleId> {
        // Check if the path already exists
        if let Ok(_) = self.resolve_path(path) {
            return Err(FsError::AlreadyExists);
        }

        // Get parent directory
        let parent_path = path.parent().ok_or(FsError::InvalidArgument)?;
        let parent_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or(FsError::InvalidName)?;

        let (parent_id, _) = self.resolve_path(parent_path)?;
        let nodes = self.nodes.lock().unwrap();
        let parent_node = nodes.get(&parent_id).ok_or(FsError::NotFound)?;

        match &parent_node.kind {
            NodeKind::Directory { children } => {
                if children.contains_key(parent_name) {
                    return Err(FsError::AlreadyExists);
                }
            }
            NodeKind::File { .. } => return Err(FsError::NotADirectory),
            NodeKind::Symlink { .. } => return Err(FsError::NotADirectory),
        }
        drop(nodes);

        // Allocate content for the file
        let content_id = self.storage.allocate(&[])?;
        let file_node_id = self.create_file_node(content_id)?;

        // Add to parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent_node.kind {
                    children.insert(parent_name.to_string(), file_node_id);
                }
            }
        }

        // Create handle
        let handle_id = self.allocate_handle_id();
        let handle = Handle {
            id: handle_id,
            node_id: file_node_id,
            position: 0,
            options: opts.clone(),
            deleted: false,
        };

        self.handles.lock().unwrap().insert(handle_id, handle);

        // Emit event
        let path_str = path.to_string_lossy().to_string();
        self.emit_event(EventKind::Created { path: path_str });

        Ok(handle_id)
    }

    pub fn open(&self, path: &Path, opts: &OpenOptions) -> FsResult<HandleId> {
        let (node_id, _) = self.resolve_path(path)?;

        // Check share mode conflicts with existing handles
        if self.share_mode_conflicts(node_id, opts) {
            return Err(FsError::AccessDenied);
        }

        // Create handle
        let handle_id = self.allocate_handle_id();
        let handle = Handle {
            id: handle_id,
            node_id,
            position: 0,
            options: opts.clone(),
            deleted: false,
        };

        self.handles.lock().unwrap().insert(handle_id, handle);
        Ok(handle_id)
    }

    /// Open by internal node id (adapter pathless open)
    pub fn open_by_id(&self, node_id_u64: u64, opts: &OpenOptions) -> FsResult<HandleId> {
        let node_id = NodeId(node_id_u64);

        // Verify node exists
        {
            let nodes = self.nodes.lock().unwrap();
            let _ = nodes.get(&node_id).ok_or(FsError::NotFound)?;
        }

        // Check share mode conflicts with existing handles
        if self.share_mode_conflicts(node_id, opts) {
            return Err(FsError::AccessDenied);
        }

        let handle_id = self.allocate_handle_id();
        let handle = Handle {
            id: handle_id,
            node_id,
            position: 0,
            options: opts.clone(),
            deleted: false,
        };
        self.handles.lock().unwrap().insert(handle_id, handle);
        Ok(handle_id)
    }

    /// Check if a node is shared between branches/snapshots
    fn is_node_shared(&self, _node_id: NodeId) -> bool {
        // For simplicity, assume all nodes need CoW for now
        true
    }

    pub fn read(&self, handle_id: HandleId, offset: u64, buf: &mut [u8]) -> FsResult<usize> {
        let handles = self.handles.lock().unwrap();
        let handle = handles.get(&handle_id).ok_or(FsError::InvalidArgument)?;

        if !handle.options.read {
            return Err(FsError::AccessDenied);
        }

        let stream_name = Self::get_stream_name(handle);
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&handle.node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::File { streams } => {
                if let Some((content_id, _)) = streams.get(stream_name) {
                    self.storage.read(*content_id, offset, buf)
                } else {
                    Err(FsError::NotFound) // Stream doesn't exist
                }
            }
            NodeKind::Directory { .. } => Err(FsError::IsADirectory),
            NodeKind::Symlink { .. } => Err(FsError::InvalidArgument), // Symlinks are not readable like files
        }
    }

    pub fn write(&self, handle_id: HandleId, offset: u64, data: &[u8]) -> FsResult<usize> {
        let mut handles = self.handles.lock().unwrap();
        let handle = handles.get_mut(&handle_id).ok_or(FsError::InvalidArgument)?;

        if !handle.options.write {
            return Err(FsError::AccessDenied);
        }

        let stream_name = Self::get_stream_name(handle);
        let current_branch_id = self.current_branch_for_process();
        let _branches = self.branches.lock().unwrap();
        let _branch = _branches.get(&current_branch_id).ok_or(FsError::NotFound)?;

        let mut nodes = self.nodes.lock().unwrap();
        let node = nodes.get_mut(&handle.node_id).ok_or(FsError::NotFound)?;

        match &mut node.kind {
            NodeKind::File { streams } => {
                // Get or create the stream
                let (content_id, size) = streams.entry(stream_name.to_string()).or_insert_with(|| {
                    // Create new stream if it doesn't exist
                    let new_content_id = self.storage.allocate(&[]).unwrap();
                    (new_content_id, 0)
                });

                let content_to_write = if self.is_content_shared(*content_id) {
                    // Clone the content for this branch
                    let new_content_id = self.storage.clone_cow(*content_id).unwrap();
                    *content_id = new_content_id;
                    new_content_id
                } else {
                    *content_id
                };

                let written = self.storage.write(content_to_write, offset, data)?;
                let new_size = std::cmp::max(*size, offset + written as u64);
                *size = new_size;
                node.times.mtime = Self::current_timestamp();
                node.times.ctime = node.times.mtime;
                Ok(written)
            }
            NodeKind::Directory { .. } => Err(FsError::IsADirectory),
            NodeKind::Symlink { .. } => Err(FsError::InvalidArgument), // Symlinks are not writable like files
        }
    }

    /// Check if content is shared between branches/snapshots
    fn is_content_shared(&self, _content_id: ContentId) -> bool {
        // For simplicity, assume all content needs CoW for now
        // In a real implementation, this would track reference counts
        true
    }

    /// Check if two lock ranges overlap
    fn ranges_overlap(r1: &LockRange, r2: &LockRange) -> bool {
        r1.offset < (r2.offset + r2.len) && r2.offset < (r1.offset + r1.len)
    }

    /// Check if a lock conflicts with existing locks
    fn lock_conflicts(&self, node_id: NodeId, new_lock: &LockRange, handle_id: HandleId) -> bool {
        let locks = self.locks.lock().unwrap();
        if let Some(node_locks) = locks.locks.get(&node_id) {
            for existing_lock in node_locks {
                // For POSIX semantics, same handle cannot have conflicting locks
                if existing_lock.handle_id == handle_id &&
                   Self::ranges_overlap(&existing_lock.range, new_lock) {
                    // Same handle: exclusive locks cannot overlap with anything
                    // Shared locks cannot overlap with exclusive locks from same handle
                    if existing_lock.range.kind == LockKind::Exclusive || new_lock.kind == LockKind::Exclusive {
                        return true;
                    }
                }

                // Different handles: check standard conflict rules
                if existing_lock.handle_id != handle_id &&
                   Self::ranges_overlap(&existing_lock.range, new_lock) {
                    // Exclusive locks conflict with any overlapping lock
                    // Shared locks only conflict with exclusive locks
                    if existing_lock.range.kind == LockKind::Exclusive || new_lock.kind == LockKind::Exclusive {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if opening with given options would conflict with existing handles (Windows share modes)
    fn share_mode_conflicts(&self, node_id: NodeId, options: &OpenOptions) -> bool {
        let handles = self.handles.lock().unwrap();

        for handle in handles.values() {
            if handle.node_id != node_id || handle.deleted {
                continue;
            }

            // Check each requested access type against existing handle's share modes
            if options.read && !handle.options.share.contains(&ShareMode::Read) {
                return true;
            }
            if options.write && !handle.options.share.contains(&ShareMode::Write) {
                return true;
            }
            // Note: Delete access conflicts are typically checked at delete time, not open time
        }

        false
    }

    /// Get the stream name for a handle (empty string for unnamed/default stream)
    fn get_stream_name(handle: &Handle) -> &str {
        handle.options.stream.as_deref().unwrap_or("")
    }

    pub fn close(&self, handle_id: HandleId) -> FsResult<()> {
        let mut handles = self.handles.lock().unwrap();
        let handle = handles.get(&handle_id).ok_or(FsError::InvalidArgument)?;
        let node_id = handle.node_id;
        let was_deleted = handle.deleted;

        handles.remove(&handle_id);

        // Clean up any locks held by this handle
        let mut locks = self.locks.lock().unwrap();
        if let Some(node_locks) = locks.locks.get_mut(&node_id) {
            node_locks.retain(|lock| lock.handle_id != handle_id);
            if node_locks.is_empty() {
                locks.locks.remove(&node_id);
            }
        }
        drop(locks);

        // If this was the last handle to a deleted file, remove the node
        if was_deleted {
            let remaining_handles: Vec<_> = handles.values()
                .filter(|h| h.node_id == node_id)
                .collect();

            if remaining_handles.is_empty() {
                let mut nodes = self.nodes.lock().unwrap();
                nodes.remove(&node_id);
            }
        }

        Ok(())
    }

    // Lock operations
    pub fn lock(&self, handle_id: HandleId, range: LockRange) -> FsResult<()> {
        let handles = self.handles.lock().unwrap();
        let handle = handles.get(&handle_id).ok_or(FsError::InvalidArgument)?;
        let node_id = handle.node_id;
        drop(handles);

        // Check for conflicts
        if self.lock_conflicts(node_id, &range, handle_id) {
            return Err(FsError::Busy); // Lock conflict
        }

        // Add the lock
        let mut locks = self.locks.lock().unwrap();
        let node_locks = locks.locks.entry(node_id).or_insert_with(Vec::new);
        node_locks.push(ActiveLock {
            handle_id,
            range,
        });

        Ok(())
    }

    pub fn unlock(&self, handle_id: HandleId, range: LockRange) -> FsResult<()> {
        let handles = self.handles.lock().unwrap();
        let handle = handles.get(&handle_id).ok_or(FsError::InvalidArgument)?;
        let node_id = handle.node_id;
        drop(handles);

        // Find and remove matching locks
        let mut locks = self.locks.lock().unwrap();
        if let Some(node_locks) = locks.locks.get_mut(&node_id) {
            // Remove locks that match the handle and range
            node_locks.retain(|lock| {
                !(lock.handle_id == handle_id &&
                  lock.range.offset == range.offset &&
                  lock.range.len == range.len &&
                  lock.range.kind == range.kind)
            });

            // Clean up empty lock lists
            if node_locks.is_empty() {
                locks.locks.remove(&node_id);
            }
        }

        Ok(())
    }

    pub fn getattr(&self, path: &Path) -> FsResult<Attributes> {
        let (node_id, _) = self.resolve_path(path)?;
        self.get_node_attributes(node_id)
    }

    pub fn set_times(&self, path: &Path, times: FileTimes) -> FsResult<()> {
        let (node_id, _) = self.resolve_path(path)?;
        self.update_node_times(node_id, times);
        Ok(())
    }

    // Directory operations
    pub fn mkdir(&self, path: &Path, _mode: u32) -> FsResult<()> {
        // Check if the path already exists
        if let Ok(_) = self.resolve_path(path) {
            return Err(FsError::AlreadyExists);
        }

        // Get parent directory
        let parent_path = path.parent().ok_or(FsError::InvalidArgument)?;
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or(FsError::InvalidName)?;

        let (parent_id, _) = self.resolve_path(parent_path)?;
        let nodes = self.nodes.lock().unwrap();
        let parent_node = nodes.get(&parent_id).ok_or(FsError::NotFound)?;

        match &parent_node.kind {
            NodeKind::Directory { children } => {
                if children.contains_key(dir_name) {
                    return Err(FsError::AlreadyExists);
                }
            }
            NodeKind::File { .. } => return Err(FsError::NotADirectory),
            NodeKind::Symlink { .. } => return Err(FsError::NotADirectory),
        }
        drop(nodes);

        // Create directory node
        let dir_node_id = self.create_directory_node()?;

        // Add to parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent_node.kind {
                    children.insert(dir_name.to_string(), dir_node_id);
                }
            }
        }

        // Emit event
        let path_str = path.to_string_lossy().to_string();
        self.emit_event(EventKind::Created { path: path_str });

        Ok(())
    }

    pub fn rmdir(&self, path: &Path) -> FsResult<()> {
        let (node_id, parent_info) = self.resolve_path(path)?;

        let Some((parent_id, name)) = parent_info else {
            return Err(FsError::InvalidArgument); // Can't remove root
        };

        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        // Check if it's a directory and empty
        match &node.kind {
            NodeKind::Directory { children } => {
                if !children.is_empty() {
                    return Err(FsError::Busy); // Directory not empty
                }
            }
            NodeKind::File { .. } => return Err(FsError::NotADirectory),
            NodeKind::Symlink { .. } => return Err(FsError::NotADirectory),
        }
        drop(nodes);

        // Remove from parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent_node.kind {
                    children.remove(&name);
                }
            }
        }

        // Remove the directory node itself to avoid leaking nodes
        {
            let mut nodes = self.nodes.lock().unwrap();
            nodes.remove(&node_id);
        }

        // Emit event
        let path_str = path.to_string_lossy().to_string();
        self.emit_event(EventKind::Removed { path: path_str });

        Ok(())
    }

    // Optional readdir+ that includes attributes without extra getattr calls (libfuse pattern)
    pub fn readdir_plus(&self, path: &Path) -> FsResult<Vec<(DirEntry, Attributes)>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::Directory { children } => {
                // Collect and sort child names for stable ordering
                let mut names: Vec<_> = children.keys().cloned().collect();
                names.sort();

                let mut entries = Vec::new();
                for name in names {
                    let child_id = children.get(&name).ok_or(FsError::NotFound)?;
                    let child_node = nodes.get(child_id).ok_or(FsError::NotFound)?;
                    let (is_dir, is_symlink, len) = match &child_node.kind {
                        NodeKind::Directory { .. } => (true, false, 0),
                        NodeKind::File { streams } => {
                            // Size is the size of the unnamed stream
                            let size = streams.get("").map(|(_, size)| *size).unwrap_or(0);
                            (false, false, size)
                        }
                        NodeKind::Symlink { target } => (false, true, target.len() as u64),
                    };

                    let dir_entry = DirEntry {
                        name: name,
                        is_dir,
                        is_symlink,
                        len,
                    };

                    let attributes = Attributes {
                        len,
                        times: child_node.times,
                        uid: child_node.uid,
                        gid: child_node.gid,
                        is_dir,
                        is_symlink,
                        mode_user: FileMode { read: true, write: true, exec: is_dir },
                        mode_group: FileMode { read: true, write: false, exec: is_dir },
                        mode_other: FileMode { read: true, write: false, exec: false },
                    };

                    entries.push((dir_entry, attributes));
                }
                Ok(entries)
            }
            NodeKind::File { .. } => Err(FsError::NotADirectory),
            NodeKind::Symlink { .. } => Err(FsError::NotADirectory),
        }
    }

    /// Like readdir_plus, but returns raw name bytes for each entry for adapters that need exact bytes
    pub fn readdir_plus_raw(&self, path: &Path) -> FsResult<Vec<(Vec<u8>, Attributes)>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::Directory { children } => {
                // Sort internal names for stable order
                let mut names: Vec<_> = children.keys().cloned().collect();
                names.sort();

                let mut entries = Vec::new();
                for name in names {
                    let child_id = children.get(&name).ok_or(FsError::NotFound)?;
                    let child_node = nodes.get(child_id).ok_or(FsError::NotFound)?;

                    let (is_dir, is_symlink, len) = match &child_node.kind {
                        NodeKind::Directory { .. } => (true, false, 0),
                        NodeKind::File { streams } => {
                            let size = streams.get("").map(|(_, size)| *size).unwrap_or(0);
                            (false, false, size)
                        }
                        NodeKind::Symlink { target } => (false, true, target.len() as u64),
                    };

                    let attributes = Attributes {
                        len,
                        times: child_node.times,
                        uid: child_node.uid,
                        gid: child_node.gid,
                        is_dir,
                        is_symlink,
                        mode_user: FileMode { read: true, write: true, exec: is_dir },
                        mode_group: FileMode { read: true, write: false, exec: is_dir },
                        mode_other: FileMode { read: true, write: false, exec: false },
                    };

                    // Prefer raw name bytes preserved at create time, fallback to internal name bytes
                    let raw_bytes = child_node
                        .xattrs
                        .get("user.agentfs.rawname")
                        .cloned()
                        .unwrap_or_else(|| name.as_bytes().to_vec());

                    entries.push((raw_bytes, attributes));
                }
                Ok(entries)
            }
            _ => Err(FsError::NotADirectory),
        }
    }

    // Extended attributes operations
    pub fn xattr_get(&self, path: &Path, name: &str) -> FsResult<Vec<u8>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;
        node.xattrs.get(name).cloned().ok_or(FsError::NotFound)
    }

    pub fn xattr_set(&self, path: &Path, name: &str, value: &[u8]) -> FsResult<()> {
        let (node_id, _) = self.resolve_path(path)?;
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(node) = nodes.get_mut(&node_id) {
            node.xattrs.insert(name.to_string(), value.to_vec());
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    pub fn xattr_list(&self, path: &Path) -> FsResult<Vec<String>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;
        Ok(node.xattrs.keys().cloned().collect())
    }

    // Alternate Data Streams operations
    pub fn streams_list(&self, path: &Path) -> FsResult<Vec<StreamSpec>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::File { streams } => {
                let mut stream_specs = Vec::new();
                for stream_name in streams.keys() {
                    if !stream_name.is_empty() { // Skip the unnamed default stream
                        stream_specs.push(StreamSpec {
                            name: stream_name.clone(),
                        });
                    }
                }
                Ok(stream_specs)
            }
            NodeKind::Directory { .. } => Err(FsError::IsADirectory),
            NodeKind::Symlink { .. } => Err(FsError::InvalidArgument), // Symlinks don't have streams
        }
    }

    pub fn unlink(&self, path: &Path) -> FsResult<()> {
        let (node_id, parent_info) = self.resolve_path(path)?;

        let Some((parent_id, name)) = parent_info else {
            return Err(FsError::InvalidArgument); // Can't unlink root
        };

        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        // Check if it's a file or symlink (can't unlink directories with unlink)
        match &node.kind {
            NodeKind::Directory { .. } => return Err(FsError::IsADirectory),
            NodeKind::File { .. } => {}
            NodeKind::Symlink { .. } => {} // Symlinks can be unlinked like files
        }
        drop(nodes);

        // Check if any handles are open to this file and mark them as deleted
        let mut handles = self.handles.lock().unwrap();
        let has_open_handles = handles.values().any(|h| h.node_id == node_id);

        if has_open_handles {
            // Mark all handles to this file as deleted
            for handle in handles.values_mut() {
                if handle.node_id == node_id {
                    handle.deleted = true;
                }
            }
        } else {
            // No open handles, remove immediately
            let mut nodes = self.nodes.lock().unwrap();
            nodes.remove(&node_id);
        }

        // Remove from parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent_node.kind {
                    children.remove(&name);
                }
            }
        }

        // Emit event
        let path_str = path.to_string_lossy().to_string();
        self.emit_event(EventKind::Removed { path: path_str });

        Ok(())
    }

    /// Public helper to resolve path and return internal IDs for FFI consumers
    pub fn resolve_path_public(&self, path: &Path) -> FsResult<(u64, Option<u64>)> {
        let (node_id, parent_info) = self.resolve_path(path)?;
        let parent_id = parent_info.map(|(pid, _name)| pid.0);
        Ok((node_id.0, parent_id))
    }

    /// Change permissions mode on a path (basic chmod semantics)
    pub fn set_mode(&self, path: &Path, mode: u32) -> FsResult<()> {
        let (node_id, _) = self.resolve_path(path)?;
        let mut nodes = self.nodes.lock().unwrap();
        let node = nodes.get_mut(&node_id).ok_or(FsError::NotFound)?;
        node.mode = mode;
        // ctime changes on metadata change
        let now = FsCore::current_timestamp();
        node.times.ctime = now;
        Ok(())
    }

    /// Rename a node from old path to new path. Fails if destination exists.
    pub fn rename(&self, old: &Path, new: &Path) -> FsResult<()> {
        // Resolve old path and its parent
        let (old_id, old_parent) = self.resolve_path(old)?;
        let Some((old_parent_id, old_name)) = old_parent else {
            return Err(FsError::InvalidArgument); // Cannot rename root
        };

        // Resolve destination parent and name
        let new_parent_path = new.parent().ok_or(FsError::InvalidArgument)?;
        let new_name = new
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(FsError::InvalidName)?
            .to_string();

        let (new_parent_id, _) = self.resolve_path(new_parent_path)?;

        // Lock nodes for mutation
        let mut nodes = self.nodes.lock().unwrap();

        // Ensure destination does not exist
        if let Some(parent_node) = nodes.get(&new_parent_id) {
            if let NodeKind::Directory { children } = &parent_node.kind {
                if children.contains_key(&new_name) {
                    return Err(FsError::AlreadyExists);
                }
            } else {
                return Err(FsError::NotADirectory);
            }
        } else {
            return Err(FsError::NotFound);
        }

        // Remove from old parent's children and insert into new parent's children
        // Also update ctime on the moved node and both directories
        let now = FsCore::current_timestamp();

        // Remove from old parent
        if let Some(old_parent_node) = nodes.get_mut(&old_parent_id) {
            if let NodeKind::Directory { children } = &mut old_parent_node.kind {
                children.remove(&old_name);
                old_parent_node.times.ctime = now;
            }
        }

        // Insert into new parent
        if let Some(new_parent_node) = nodes.get_mut(&new_parent_id) {
            if let NodeKind::Directory { children } = &mut new_parent_node.kind {
                children.insert(new_name, old_id);
                new_parent_node.times.ctime = now;
            }
        }

        // Update moved node's ctime
        if let Some(node) = nodes.get_mut(&old_id) {
            node.times.ctime = now;
        }

        Ok(())
    }

    /// Create a symbolic link
    pub fn symlink(&self, target: &str, linkpath: &Path) -> FsResult<()> {
        // Check if the link path already exists
        if self.resolve_path(linkpath).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        // Resolve parent directory
        let parent_path = linkpath.parent().ok_or(FsError::InvalidArgument)?;
        let link_name = linkpath.file_name()
            .ok_or(FsError::InvalidArgument)?
            .to_string_lossy()
            .to_string();

        let (parent_id, _) = self.resolve_path(parent_path)?;
        let nodes = self.nodes.lock().unwrap();

        // Check that parent is a directory
        if let Some(parent_node) = nodes.get(&parent_id) {
            match &parent_node.kind {
                NodeKind::Directory { children } => {
                    if children.contains_key(&link_name) {
                        return Err(FsError::AlreadyExists);
                    }
                }
                _ => return Err(FsError::NotADirectory),
            }
        } else {
            return Err(FsError::NotFound);
        }
        drop(nodes);

        // Create symlink node
        let symlink_node_id = self.create_symlink_node(target.to_string())?;

        // Add to parent directory
        {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(parent_node) = nodes.get_mut(&parent_id) {
                if let NodeKind::Directory { children } = &mut parent_node.kind {
                    children.insert(link_name, symlink_node_id);
                }
            }
        }

        // Emit event
        let path_str = linkpath.to_string_lossy().to_string();
        self.emit_event(EventKind::Created { path: path_str });

        Ok(())
    }

    /// Read a symbolic link
    pub fn readlink(&self, path: &Path) -> FsResult<String> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::Symlink { target } => Ok(target.clone()),
            _ => Err(FsError::InvalidArgument), // Not a symlink
        }
    }

    /// Create a symlink node
    fn create_symlink_node(&self, target: String) -> FsResult<NodeId> {
        let now = Self::current_timestamp();
        let node_id = self.allocate_node_id();

        let node = Node {
            id: node_id,
            kind: NodeKind::Symlink { target },
            times: FileTimes {
                atime: now,
                mtime: now,
                ctime: now,
                birthtime: now,
            },
            mode: 0o777, // Symlinks typically have full permissions
            uid: self.config.security.default_uid,
            gid: self.config.security.default_gid,
            xattrs: HashMap::new(),
        };

        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node_id, node);

        Ok(node_id)
    }
}
