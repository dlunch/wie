use alloc::string::String;
use core::cmp::min;
use std::collections::HashMap;

#[derive(Default)]
pub struct Filesystem {
    virtual_files: HashMap<String, Vec<u8>>,
}

impl Filesystem {
    pub fn new() -> Self {
        Self {
            virtual_files: HashMap::new(),
        }
    }

    pub fn add(&mut self, path: &str, data: Vec<u8>) {
        self.virtual_files.insert(path.to_string(), data);
    }

    pub fn exists(&self, path: &str) -> bool {
        let normalized_path = Self::normalize_path(path);

        self.virtual_files.contains_key(normalized_path)
    }

    pub fn size(&self, path: &str) -> Option<usize> {
        let normalized_path = Self::normalize_path(path);

        if let Some(data) = self.virtual_files.get(normalized_path) {
            return Some(data.len());
        }

        None
    }

    pub fn read(&self, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let normalized_path = Self::normalize_path(path);

        if let Some(data) = self.virtual_files.get(normalized_path) {
            let data_size = data.len();
            let size_to_read = min(count, data_size - offset);

            buf[..size_to_read].copy_from_slice(&data[offset..offset + size_to_read]);

            return Some(size_to_read);
        }

        None
    }

    pub fn write(&mut self, path: &str, offset: usize, data: &[u8]) -> usize {
        let normalized_path = Self::normalize_path(path);

        let file = self.virtual_files.get_mut(normalized_path).unwrap();
        if file.len() < offset + data.len() {
            file.resize(offset + data.len(), 0);
        }
        file[offset..offset + data.len()].copy_from_slice(data);

        data.len()
    }

    pub fn truncate(&mut self, path: &str, len: usize) {
        let normalized_path = Self::normalize_path(path);

        if let Some(data) = self.virtual_files.get_mut(normalized_path) {
            data.resize(len, 0);
        }
    }

    pub fn files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.virtual_files.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    fn normalize_path(path: &str) -> &str {
        path.trim_start_matches('/')
    }
}
