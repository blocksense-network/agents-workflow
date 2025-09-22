//! Virtual filesystem implementation for AgentFS Core

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::os::unix::process::parent_id;

use crate::{
    Attributes, BranchId, BranchInfo, ContentId, DirEntry, FileMode, FileTimes, FsConfig, HandleId, OpenOptions, SnapshotId,
};
use crate::error::{FsError, FsResult};
use crate::storage::StorageBackend;

/// Internal node ID for filesystem nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(u64);

/// Filesystem node types
#[derive(Clone, Debug)]
pub(crate) enum NodeKind {
    File { content_id: ContentId, size: u64 },
    Directory { children: HashMap<String, NodeId> },
}

/// Filesystem node
#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub times: FileTimes,
    pub mode: u32,
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
    pub(crate) process_branches: Mutex<HashMap<u32, BranchId>>, // Process ID -> Branch ID mapping
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
            process_branches: Mutex::new(HashMap::new()), // No processes initially bound
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
            }
        }

        Ok((current_node_id, parent_node_id.zip(parent_name)))
    }


    /// Create a new file node
    fn create_file_node(&self, content_id: ContentId) -> FsResult<NodeId> {
        let node_id = self.allocate_node_id();
        let now = Self::current_timestamp();

        let node = Node {
            id: node_id,
            kind: NodeKind::File {
                content_id,
                size: 0,
            },
            times: FileTimes {
                atime: now,
                mtime: now,
                ctime: now,
                birthtime: now,
            },
            mode: 0o644,
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
        };

        self.nodes.lock().unwrap().insert(node_id, node);
        Ok(node_id)
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
        let mut new_node = node;

        // For files, we need to clone the content in storage
        if let NodeKind::File { content_id, size } = &new_node.kind {
            let new_content_id = self.storage.clone_cow(*content_id)?;
            new_node.kind = NodeKind::File {
                content_id: new_content_id,
                size: *size,
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

        let (len, is_dir) = match &node.kind {
            NodeKind::File { size, .. } => (*size, false),
            NodeKind::Directory { .. } => (0, true),
        };

        Ok(Attributes {
            len,
            times: node.times,
            is_dir,
            is_symlink: false, // TODO: Implement symlinks
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
        Ok(handle_id)
    }

    pub fn open(&self, path: &Path, opts: &OpenOptions) -> FsResult<HandleId> {
        let (node_id, _) = self.resolve_path(path)?;

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

        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&handle.node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::File { content_id, .. } => {
                self.storage.read(*content_id, offset, buf)
            }
            NodeKind::Directory { .. } => Err(FsError::IsADirectory),
        }
    }

    pub fn write(&self, handle_id: HandleId, offset: u64, data: &[u8]) -> FsResult<usize> {
        let mut handles = self.handles.lock().unwrap();
        let handle = handles.get_mut(&handle_id).ok_or(FsError::InvalidArgument)?;

        if !handle.options.write {
            return Err(FsError::AccessDenied);
        }

        let current_branch_id = self.current_branch_for_process();
        let branches = self.branches.lock().unwrap();
        let branch = branches.get(&current_branch_id).ok_or(FsError::NotFound)?;

        let mut nodes = self.nodes.lock().unwrap();
        let node = nodes.get_mut(&handle.node_id).ok_or(FsError::NotFound)?;

        match &mut node.kind {
            NodeKind::File { content_id, size } => {
                let content_to_write = if self.is_content_shared(*content_id) {
                    // Clone the content for this branch
                    let new_content_id = self.storage.clone_cow(*content_id)?;
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
        }
    }

    /// Check if content is shared between branches/snapshots
    fn is_content_shared(&self, _content_id: ContentId) -> bool {
        // For simplicity, assume all content needs CoW for now
        // In a real implementation, this would track reference counts
        true
    }

    pub fn close(&self, handle_id: HandleId) -> FsResult<()> {
        let mut handles = self.handles.lock().unwrap();
        let handle = handles.get(&handle_id).ok_or(FsError::InvalidArgument)?;
        let node_id = handle.node_id;
        let was_deleted = handle.deleted;

        handles.remove(&handle_id);

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
            // Remove the directory node itself
            nodes.remove(&node_id);
        }

        Ok(())
    }

    pub fn readdir(&self, path: &Path) -> FsResult<Vec<DirEntry>> {
        let (node_id, _) = self.resolve_path(path)?;
        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        match &node.kind {
            NodeKind::Directory { children } => {
                let mut entries = Vec::new();
                for (name, child_id) in children {
                    let child_node = nodes.get(child_id).ok_or(FsError::NotFound)?;
                    let (is_dir, len) = match &child_node.kind {
                        NodeKind::Directory { .. } => (true, 0),
                        NodeKind::File { size, .. } => (false, *size),
                    };
                    entries.push(DirEntry {
                        name: name.clone(),
                        is_dir,
                        is_symlink: false, // TODO: Implement symlinks
                        len,
                    });
                }
                Ok(entries)
            }
            NodeKind::File { .. } => Err(FsError::NotADirectory),
        }
    }

    pub fn unlink(&self, path: &Path) -> FsResult<()> {
        let (node_id, parent_info) = self.resolve_path(path)?;

        let Some((parent_id, name)) = parent_info else {
            return Err(FsError::InvalidArgument); // Can't unlink root
        };

        let nodes = self.nodes.lock().unwrap();
        let node = nodes.get(&node_id).ok_or(FsError::NotFound)?;

        // Check if it's a file (can't unlink directories with unlink)
        match &node.kind {
            NodeKind::Directory { .. } => return Err(FsError::IsADirectory),
            NodeKind::File { .. } => {}
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

        Ok(())
    }
}
