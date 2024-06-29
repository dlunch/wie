use alloc::string::String;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use zip::ZipArchive;

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
        if let Some(data) = self.virtual_files.get(path) {
            return Some(data);
        }

        None
    }

    pub fn files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.virtual_files.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    pub fn mount_zip(&mut self, zip: &[u8]) {
        let mut archive = ZipArchive::new(Cursor::new(zip)).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();

            self.add(file.name(), data);
        }
    }
}
