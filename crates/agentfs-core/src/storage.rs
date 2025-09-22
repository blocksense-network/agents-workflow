//! Storage backend implementations for AgentFS Core

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{ContentId, FsError};
use crate::error::FsResult;

/// Storage backend trait for content-addressable storage with copy-on-write
pub trait StorageBackend: Send + Sync {
    fn read(&self, id: ContentId, offset: u64, buf: &mut [u8]) -> FsResult<usize>;
    fn write(&self, id: ContentId, offset: u64, data: &[u8]) -> FsResult<usize>;
    fn truncate(&self, id: ContentId, new_len: u64) -> FsResult<()>;
    fn allocate(&self, initial: &[u8]) -> FsResult<ContentId>;
    fn clone_cow(&self, base: ContentId) -> FsResult<ContentId>;
    fn seal(&self, id: ContentId) -> FsResult<()>; // for snapshot immutability
}

/// In-memory storage backend implementation
pub struct InMemoryBackend {
    next_id: Mutex<u64>,
    data: Mutex<HashMap<ContentId, Vec<u8>>>,
    refcounts: Mutex<HashMap<ContentId, usize>>,
    sealed: Mutex<HashMap<ContentId, bool>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            next_id: Mutex::new(1),
            data: Mutex::new(HashMap::new()),
            refcounts: Mutex::new(HashMap::new()),
            sealed: Mutex::new(HashMap::new()),
        }
    }

    fn get_next_id(&self) -> ContentId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = ContentId::new(*next_id);
        *next_id += 1;
        id
    }

    fn increment_refcount(&self, id: ContentId) {
        let mut refcounts = self.refcounts.lock().unwrap();
        *refcounts.entry(id).or_insert(0) += 1;
    }

    fn decrement_refcount(&self, id: ContentId) {
        let mut refcounts = self.refcounts.lock().unwrap();
        if let Some(count) = refcounts.get_mut(&id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                refcounts.remove(&id);
                let mut data = self.data.lock().unwrap();
                data.remove(&id);
                let mut sealed = self.sealed.lock().unwrap();
                sealed.remove(&id);
            }
        }
    }
}

impl StorageBackend for InMemoryBackend {
    fn read(&self, id: ContentId, offset: u64, buf: &mut [u8]) -> FsResult<usize> {
        let data = self.data.lock().unwrap();
        let content = data.get(&id).ok_or(FsError::NotFound)?;

        let start = offset as usize;
        if start >= content.len() {
            return Ok(0);
        }

        let end = std::cmp::min(start + buf.len(), content.len());
        let bytes_to_copy = end - start;
        buf[..bytes_to_copy].copy_from_slice(&content[start..end]);
        Ok(bytes_to_copy)
    }

    fn write(&self, id: ContentId, offset: u64, data: &[u8]) -> FsResult<usize> {
        let mut storage_data = self.data.lock().unwrap();
        let content = storage_data.get_mut(&id).ok_or(FsError::NotFound)?;

        let start = offset as usize;
        let end = start + data.len();

        // Extend the content if necessary
        if end > content.len() {
            content.resize(end, 0);
        }

        content[start..end].copy_from_slice(data);
        Ok(data.len())
    }

    fn truncate(&self, id: ContentId, new_len: u64) -> FsResult<()> {
        let mut data = self.data.lock().unwrap();
        let content = data.get_mut(&id).ok_or(FsError::NotFound)?;
        content.resize(new_len as usize, 0);
        Ok(())
    }

    fn allocate(&self, initial: &[u8]) -> FsResult<ContentId> {
        let id = self.get_next_id();
        let mut data = self.data.lock().unwrap();
        data.insert(id, initial.to_vec());
        self.increment_refcount(id);
        Ok(id)
    }

    fn clone_cow(&self, base: ContentId) -> FsResult<ContentId> {
        let base_content = {
            let data = self.data.lock().unwrap();
            data.get(&base).ok_or(FsError::NotFound)?.clone()
        };
        let id = self.get_next_id();
        {
            let mut data_mut = self.data.lock().unwrap();
            data_mut.insert(id, base_content);
        }
        self.increment_refcount(id);
        Ok(id)
    }

    fn seal(&self, id: ContentId) -> FsResult<()> {
        let data = self.data.lock().unwrap();
        if !data.contains_key(&id) {
            return Err(FsError::NotFound);
        }

        let mut sealed = self.sealed.lock().unwrap();
        sealed.insert(id, true);
        Ok(())
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_backend_basic() {
        let backend = InMemoryBackend::new();

        // Allocate some content
        let id = backend.allocate(b"hello world").unwrap();
        assert_eq!(backend.data.lock().unwrap().get(&id).unwrap().as_slice(), b"hello world");

        // Read it back
        let mut buf = [0u8; 5];
        let n = backend.read(id, 0, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf, b"hello");

        // Write to it
        let n = backend.write(id, 6, b"AgentFS").unwrap();
        assert_eq!(n, 7);

        // Read the modified content
        let mut buf = [0u8; 13];
        let n = backend.read(id, 0, &mut buf).unwrap();
        assert_eq!(n, 13);
        assert_eq!(&buf, b"hello AgentFS");

        // Truncate
        backend.truncate(id, 5).unwrap();
        let mut buf = [0u8; 10];
        let n = backend.read(id, 0, &mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf[..5], b"hello");
    }

    #[test]
    fn test_clone_cow() {
        let backend = InMemoryBackend::new();

        let id1 = backend.allocate(b"original").unwrap();
        let id2 = backend.clone_cow(id1).unwrap();

        // They should have the same content
        let mut buf1 = [0u8; 8];
        let mut buf2 = [0u8; 8];
        backend.read(id1, 0, &mut buf1).unwrap();
        backend.read(id2, 0, &mut buf2).unwrap();
        assert_eq!(&buf1, &buf2);
        assert_eq!(&buf1, b"original");

        // Modifying one shouldn't affect the other
        backend.write(id2, 0, b"modified").unwrap();

        let mut buf1 = [0u8; 8];
        let mut buf2 = [0u8; 8];
        backend.read(id1, 0, &mut buf1).unwrap();
        backend.read(id2, 0, &mut buf2).unwrap();
        assert_eq!(&buf1, b"original");
        assert_eq!(&buf2, b"modified");
    }

    #[test]
    fn test_seal() {
        let backend = InMemoryBackend::new();
        let id = backend.allocate(b"test").unwrap();

        // Should be able to write before sealing
        backend.write(id, 0, b"modified").unwrap();

        // Seal it
        backend.seal(id).unwrap();

        // Verify it's marked as sealed
        assert_eq!(*backend.sealed.lock().unwrap().get(&id).unwrap(), true);
    }

    #[test]
    fn test_read_beyond_eof() {
        let backend = InMemoryBackend::new();
        let id = backend.allocate(b"short").unwrap();

        let mut buf = [0u8; 10];
        let n = backend.read(id, 10, &mut buf).unwrap();
        assert_eq!(n, 0); // Should return 0 bytes when reading beyond EOF
    }
}
