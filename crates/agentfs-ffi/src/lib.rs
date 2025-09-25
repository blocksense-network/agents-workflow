//! AgentFS FFI — C ABI for FSKit and other integrations
//!
//! This crate provides a C-compatible ABI for integrating AgentFS
//! with platform-specific filesystem frameworks like FSKit (macOS).

pub mod c_api;

#[cfg(test)]
mod tests {
    use super::c_api::*;
    use std::ffi::CString;

    fn create_fs() -> u64 {
        let cfg = CString::new("{\"max_memory_bytes\": 1048576, \"max_open_handles\": 128, \"max_branches\": 8, \"max_snapshots\": 8}").unwrap();
        let mut fs: u64 = 0;
        let rc = unsafe { af_fs_create(cfg.as_ptr(), &mut fs as *mut u64) } as i32;
        assert_eq!(rc, 0);
        fs
    }

    #[test]
    fn test_basic_readdir_and_attrs() {
        let fs = create_fs();
        let path = CString::new("/").unwrap();
        let mut buf = vec![0u8; 4096];
        let mut out_len: usize = 0;
        let rc = unsafe { af_readdir(fs, path.as_ptr(), buf.as_mut_ptr(), buf.len(), &mut out_len as *mut usize) } as i32;
        assert_eq!(rc, 0);

        // getattr root
        let mut abuf = vec![0u8; 64];
        let rc = unsafe { af_getattr(fs, path.as_ptr(), abuf.as_mut_ptr(), abuf.len()) } as i32;
        assert_eq!(rc, 0);
    }

    #[test]
    fn test_set_mode_times_rename_and_xattr() {
        let fs = create_fs();
        let fname = CString::new("/file").unwrap();
        // Create via open(create=true)
        let opts = CString::new("{\"read\":true,\"write\":true,\"create\":true,\"truncate\":true}").unwrap();
        let mut h: u64 = 0;
        let rc = unsafe { af_open(fs, fname.as_ptr(), opts.as_ptr(), &mut h as *mut u64) } as i32;
        assert_eq!(rc, 0);
        unsafe { af_close(fs, h) };

        // chmod
        let rc = unsafe { af_set_mode(fs, fname.as_ptr(), 0o600) } as i32;
        assert_eq!(rc, 0);
        // utimens
        let rc = unsafe { af_set_times(fs, fname.as_ptr(), 10, 10, 10, 1) } as i32;
        assert_eq!(rc, 0);

        // xattr set/get/list
        let key = CString::new("user.test").unwrap();
        let val = b"value";
        let rc = unsafe { af_xattr_set(fs, fname.as_ptr(), key.as_ptr(), val.as_ptr(), val.len()) } as i32;
        assert_eq!(rc, 0);
        let mut vbuf = vec![0u8; 32];
        let mut vlen: usize = 0;
        let rc = unsafe { af_xattr_get(fs, fname.as_ptr(), key.as_ptr(), vbuf.as_mut_ptr(), vbuf.len(), &mut vlen as *mut usize) } as i32;
        assert_eq!(rc, 0);
        assert_eq!(&vbuf[..vlen], val);
        let mut lbuf = vec![0u8; 64];
        let mut llen: usize = 0;
        let rc = unsafe { af_xattr_list(fs, fname.as_ptr(), lbuf.as_mut_ptr(), lbuf.len(), &mut llen as *mut usize) } as i32;
        assert_eq!(rc, 0);

        // rename
        let newname = CString::new("/file2").unwrap();
        let rc = unsafe { af_rename(fs, fname.as_ptr(), newname.as_ptr()) } as i32;
        assert_eq!(rc, 0);
    }
}

// Re-export C API functions
pub use c_api::*;
