use alloc::string::String;
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

    pub fn read(&self, path: &str) -> Option<&[u8]> {
        let normalized_path = path.trim_start_matches('/');

        if let Some(data) = self.virtual_files.get(normalized_path) {
            return Some(data);
        }

        None
    }

    pub fn files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.virtual_files.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }
}
