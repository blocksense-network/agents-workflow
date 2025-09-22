//! C API definitions for AgentFS FFI

use agentfs_core::{FsCore, FsConfig, CaseSensitivity, MemoryPolicy, FsLimits, CachePolicy, OpenOptions};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Mutex;
use std::ptr;

// Global registry for filesystem instances
lazy_static::lazy_static! {
    static ref FS_INSTANCES: Mutex<HashMap<u64, FsCore>> = Mutex::new(HashMap::new());
    static ref NEXT_ID: Mutex<u64> = Mutex::new(1);
}

/// Opaque filesystem handle (just an ID)
pub type AfFs = u64;

/// Snapshot ID (16 bytes)
#[repr(C)]
pub struct AfSnapshotId {
    pub bytes: [u8; 16],
}

/// Branch ID (16 bytes)
#[repr(C)]
pub struct AfBranchId {
    pub bytes: [u8; 16],
}

/// Handle ID
pub type AfHandleId = u64;

/// Result codes matching POSIX errno where applicable
#[repr(C)]
pub enum AfResult {
    AfOk = 0,
    AfErrNotFound = 2,     // ENOENT
    AfErrExists = 17,      // EEXIST
    AfErrAcces = 13,       // EACCES
    AfErrNospc = 28,       // ENOSPC
    AfErrInval = 22,       // EINVAL
    AfErrBusy = 16,        // EBUSY
    AfErrUnsupported = 95, // ENOTSUP
}

/// Lifecycle functions
#[no_mangle]
pub extern "C" fn af_fs_create(config_json: *const c_char, out_fs: *mut AfFs) -> AfResult {
    if config_json.is_null() || out_fs.is_null() {
        return AfResult::AfErrInval;
    }

    // Parse config JSON
    let c_str = unsafe { CStr::from_ptr(config_json) };
    let config_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return AfResult::AfErrInval,
    };

    let config: serde_json::Value = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(_) => return AfResult::AfErrInval,
    };

    // Create FsConfig from JSON
    let fs_config = FsConfig {
        case_sensitivity: CaseSensitivity::InsensitivePreserving, // Default for macOS
        memory: MemoryPolicy {
            max_bytes_in_memory: config.get("max_memory_bytes")
                .and_then(|v| v.as_u64()),
            spill_directory: config.get("spill_directory")
                .and_then(|v| v.as_str())
                .map(|s| s.into()),
        },
        limits: FsLimits {
            max_open_handles: config.get("max_open_handles")
                .and_then(|v| v.as_u64())
                .unwrap_or(65536) as u32,
            max_branches: config.get("max_branches")
                .and_then(|v| v.as_u64())
                .unwrap_or(256) as u32,
            max_snapshots: config.get("max_snapshots")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as u32,
        },
        cache: CachePolicy {
            attr_ttl_ms: 1000,
            entry_ttl_ms: 1000,
            negative_ttl_ms: 1000,
            enable_readdir_plus: true,
            auto_cache: true,
            writeback_cache: false,
        },
        enable_xattrs: true,
        enable_ads: false, // macOS uses xattrs instead of ADS
        track_events: true,
    };

    // Create FsCore instance
    let fs_core = match FsCore::new(fs_config) {
        Ok(core) => core,
        Err(_) => return AfResult::AfErrInval,
    };

    // Store in registry and return ID
    let mut instances = FS_INSTANCES.lock().unwrap();
    let mut next_id = NEXT_ID.lock().unwrap();
    let id = *next_id;
    *next_id += 1;

    instances.insert(id, fs_core);
    unsafe { *out_fs = id };

    AfResult::AfOk
}

#[no_mangle]
pub extern "C" fn af_fs_destroy(fs: AfFs) -> AfResult {
    let mut instances = FS_INSTANCES.lock().unwrap();
    instances.remove(&fs);
    AfResult::AfOk
}


/// Convert FsError to AfResult
fn fs_error_to_af_result(err: &agentfs_core::FsError) -> AfResult {
    match err {
        agentfs_core::FsError::NotFound => AfResult::AfErrNotFound,
        agentfs_core::FsError::AlreadyExists => AfResult::AfErrExists,
        agentfs_core::FsError::AccessDenied => AfResult::AfErrAcces,
        agentfs_core::FsError::InvalidArgument => AfResult::AfErrInval,
        agentfs_core::FsError::Busy => AfResult::AfErrBusy,
        agentfs_core::FsError::NoSpace => AfResult::AfErrNospc,
        agentfs_core::FsError::Unsupported => AfResult::AfErrUnsupported,
        _ => AfResult::AfErrInval,
    }
}

/// Snapshot operations
#[no_mangle]
pub extern "C" fn af_snapshot_create(
    fs: AfFs,
    name: *const c_char,
    out_id: *mut AfSnapshotId,
) -> AfResult {
    if out_id.is_null() {
        return AfResult::AfErrInval;
    }

    let name_str = if name.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => Some(s),
            Err(_) => return AfResult::AfErrInval,
        }
    };

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    match core.snapshot_create(name_str) {
        Ok(snapshot_id) => {
            unsafe { (*out_id).bytes.copy_from_slice(&snapshot_id.0) };
            AfResult::AfOk
        }
        Err(e) => fs_error_to_af_result(&e),
    }
}

/// Branch operations
#[no_mangle]
pub extern "C" fn af_branch_create_from_snapshot(
    fs: AfFs,
    snap: AfSnapshotId,
    name: *const c_char,
    out_id: *mut AfBranchId,
) -> AfResult {
    if out_id.is_null() {
        return AfResult::AfErrInval;
    }

    let name_str = if name.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => Some(s),
            Err(_) => return AfResult::AfErrInval,
        }
    };

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    let snapshot_id = agentfs_core::SnapshotId(snap.bytes);

    match core.branch_create_from_snapshot(snapshot_id, name_str) {
        Ok(branch_id) => {
            unsafe { (*out_id).bytes.copy_from_slice(&branch_id.0) };
            AfResult::AfOk
        }
        Err(e) => fs_error_to_af_result(&e),
    }
}

#[no_mangle]
pub extern "C" fn af_bind_process_to_branch(fs: AfFs, branch: AfBranchId) -> AfResult {
    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    let branch_id = agentfs_core::BranchId(branch.bytes);

    match core.bind_process_to_branch(branch_id) {
        Ok(()) => AfResult::AfOk,
        Err(e) => fs_error_to_af_result(&e),
    }
}

/// File operations (minimal set)
#[no_mangle]
pub extern "C" fn af_mkdir(fs: AfFs, path: *const c_char, mode: u32) -> AfResult {
    if path.is_null() {
        return AfResult::AfErrInval;
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return AfResult::AfErrInval,
    };

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    match core.mkdir(Path::new(path_str), mode) {
        Ok(()) => AfResult::AfOk,
        Err(e) => fs_error_to_af_result(&e),
    }
}

#[no_mangle]
pub extern "C" fn af_open(
    fs: AfFs,
    path: *const c_char,
    options_json: *const c_char,
    out_h: *mut AfHandleId,
) -> AfResult {
    if path.is_null() || options_json.is_null() || out_h.is_null() {
        return AfResult::AfErrInval;
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return AfResult::AfErrInval,
    };

    let options_str = match unsafe { CStr::from_ptr(options_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return AfResult::AfErrInval,
    };

    // Parse options JSON - simplified for now
    let options: serde_json::Value = match serde_json::from_str(options_str) {
        Ok(o) => o,
        Err(_) => return AfResult::AfErrInval,
    };

    let open_options = OpenOptions {
        read: options.get("read").and_then(|v| v.as_bool()).unwrap_or(false),
        write: options.get("write").and_then(|v| v.as_bool()).unwrap_or(false),
        create: options.get("create").and_then(|v| v.as_bool()).unwrap_or(false),
        truncate: options.get("truncate").and_then(|v| v.as_bool()).unwrap_or(false),
        append: options.get("append").and_then(|v| v.as_bool()).unwrap_or(false),
        share: vec![], // Simplified
        stream: None,  // macOS uses xattrs instead
    };

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    let result = if open_options.create {
        core.create(Path::new(path_str), &open_options)
    } else {
        core.open(Path::new(path_str), &open_options)
    };

    match result {
        Ok(handle_id) => {
            unsafe { *out_h = handle_id.0 };
            AfResult::AfOk
        }
        Err(e) => fs_error_to_af_result(&e),
    }
}

#[no_mangle]
pub extern "C" fn af_read(
    fs: AfFs,
    h: AfHandleId,
    off: u64,
    buf: *mut u8,
    len: u32,
    out_read: *mut u32,
) -> AfResult {
    if buf.is_null() || out_read.is_null() {
        return AfResult::AfErrInval;
    }

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    let buffer = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };

    match core.read(agentfs_core::HandleId(h), off, buffer) {
        Ok(bytes_read) => {
            unsafe { *out_read = bytes_read as u32 };
            AfResult::AfOk
        }
        Err(e) => fs_error_to_af_result(&e),
    }
}

#[no_mangle]
pub extern "C" fn af_write(
    fs: AfFs,
    h: AfHandleId,
    off: u64,
    buf: *const u8,
    len: u32,
    out_written: *mut u32,
) -> AfResult {
    if buf.is_null() || out_written.is_null() {
        return AfResult::AfErrInval;
    }

    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    let buffer = unsafe { std::slice::from_raw_parts(buf, len as usize) };

    match core.write(agentfs_core::HandleId(h), off, buffer) {
        Ok(bytes_written) => {
            unsafe { *out_written = bytes_written as u32 };
            AfResult::AfOk
        }
        Err(e) => fs_error_to_af_result(&e),
    }
}

#[no_mangle]
pub extern "C" fn af_close(fs: AfFs, h: AfHandleId) -> AfResult {
    let instances = FS_INSTANCES.lock().unwrap();
    let core = match instances.get(&fs) {
        Some(c) => c,
        None => return AfResult::AfErrInval,
    };

    match core.close(agentfs_core::HandleId(h)) {
        Ok(()) => AfResult::AfOk,
        Err(e) => fs_error_to_af_result(&e),
    }
}
