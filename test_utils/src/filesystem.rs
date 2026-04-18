use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::cmp::min;

use hashbrown::HashMap;
use spin::Mutex;

use wie_backend::Filesystem;

/// In-memory `Filesystem` implementation for tests.
#[derive(Default)]
pub struct MemoryFilesystem {
    files: Mutex<HashMap<(String, String), Vec<u8>>>,
}

impl MemoryFilesystem {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Filesystem for MemoryFilesystem {
    fn exists(&self, aid: &str, path: &str) -> bool {
        self.files.lock().contains_key(&(aid.to_string(), path.to_string()))
    }

    fn size(&self, aid: &str, path: &str) -> Option<usize> {
        self.files.lock().get(&(aid.to_string(), path.to_string())).map(|v| v.len())
    }

    fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let files = self.files.lock();
        let data = files.get(&(aid.to_string(), path.to_string()))?;

        if offset >= data.len() {
            return Some(0);
        }

        let size_to_read = min(count, data.len() - offset);
        buf[..size_to_read].copy_from_slice(&data[offset..offset + size_to_read]);
        Some(size_to_read)
    }

    fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let mut files = self.files.lock();
        let file = files.entry((aid.to_string(), path.to_string())).or_default();
        if file.len() < offset + data.len() {
            file.resize(offset + data.len(), 0);
        }
        file[offset..offset + data.len()].copy_from_slice(data);

        data.len()
    }

    fn truncate(&self, aid: &str, path: &str, len: usize) {
        let mut files = self.files.lock();
        let file = files.entry((aid.to_string(), path.to_string())).or_default();
        file.resize(len, 0);
    }
}
