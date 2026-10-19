use alloc::{
    boxed::Box,
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

#[async_trait::async_trait]
impl Filesystem for MemoryFilesystem {
    async fn remove(&self, pid: &str, path: &str) -> bool {
        self.files.lock().remove(&(pid.to_string(), path.to_string())).is_some()
    }

    async fn usage(&self, pid: &str) -> u64 {
        self.files
            .lock()
            .iter()
            .filter(|((owner, _), _)| owner == pid)
            .map(|(_, data)| data.len() as u64)
            .sum()
    }

    async fn exists(&self, pid: &str, path: &str) -> bool {
        self.files.lock().contains_key(&(pid.to_string(), path.to_string()))
    }

    async fn size(&self, pid: &str, path: &str) -> Option<usize> {
        self.files.lock().get(&(pid.to_string(), path.to_string())).map(|v| v.len())
    }

    async fn read(&self, pid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let files = self.files.lock();
        let data = files.get(&(pid.to_string(), path.to_string()))?;

        if offset >= data.len() {
            return Some(0);
        }

        let size_to_read = min(count, data.len() - offset);
        buf[..size_to_read].copy_from_slice(&data[offset..offset + size_to_read]);
        Some(size_to_read)
    }

    async fn write(&self, pid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let mut files = self.files.lock();
        let file = files.entry((pid.to_string(), path.to_string())).or_default();
        if file.len() < offset + data.len() {
            file.resize(offset + data.len(), 0);
        }
        file[offset..offset + data.len()].copy_from_slice(data);

        data.len()
    }

    async fn truncate(&self, pid: &str, path: &str, len: usize) {
        let mut files = self.files.lock();
        let file = files.entry((pid.to_string(), path.to_string())).or_default();
        file.resize(len, 0);
    }
}
