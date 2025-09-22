//! C API definitions for AgentFS FFI

use std::os::raw::c_char;

/// Opaque filesystem handle
pub type AfFs = std::ffi::c_void;

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
pub extern "C" fn af_fs_create(_config_json: *const c_char, _out_fs: *mut *mut AfFs) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_fs_destroy(_fs: *mut AfFs) -> AfResult {
    // TODO: Implement
    AfResult::AfOk
}

/// Snapshot operations
#[no_mangle]
pub extern "C" fn af_snapshot_create(
    _fs: *mut AfFs,
    _name: *const c_char,
    _out_id: *mut AfSnapshotId,
) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

/// Branch operations
#[no_mangle]
pub extern "C" fn af_branch_create_from_snapshot(
    _fs: *mut AfFs,
    _snap: AfSnapshotId,
    _name: *const c_char,
    _out_id: *mut AfBranchId,
) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_bind_process_to_branch(_fs: *mut AfFs, _branch: AfBranchId) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

/// File operations (minimal set)
#[no_mangle]
pub extern "C" fn af_mkdir(_fs: *mut AfFs, _path: *const c_char, _mode: u32) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_open(
    _fs: *mut AfFs,
    _path: *const c_char,
    _options_json: *const c_char,
    _out_h: *mut AfHandleId,
) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_read(
    _fs: *mut AfFs,
    _h: AfHandleId,
    _off: u64,
    _buf: *mut u8,
    _len: u32,
    _out_read: *mut u32,
) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_write(
    _fs: *mut AfFs,
    _h: AfHandleId,
    _off: u64,
    _buf: *const u8,
    _len: u32,
    _out_written: *mut u32,
) -> AfResult {
    // TODO: Implement
    AfResult::AfErrUnsupported
}

#[no_mangle]
pub extern "C" fn af_close(_fs: *mut AfFs, _h: AfHandleId) -> AfResult {
    // TODO: Implement
    AfResult::AfOk
}
