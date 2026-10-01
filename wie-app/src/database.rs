use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use wie_backend::{Database, DatabaseRepository as BackendDatabaseRepository, RecordId};

pub(crate) struct DatabaseRepository {
    base_path: PathBuf,
}

impl DatabaseRepository {
    pub(crate) fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn path_for(&self, name: &str, app_id: &str) -> PathBuf {
        let name: String = name.chars().map(|c| if matches!(c, '\\' | '\0' | ':') { '_' } else { c }).collect();
        let mut normalized = PathBuf::new();
        for segment in name.trim_start_matches('/').split('/') {
            match segment {
                "" | "." => {}
                ".." => normalized.push("_"),
                segment => normalized.push(segment),
            }
        }
        if normalized.as_os_str().is_empty() {
            normalized.push("_");
        }

        self.base_path.join(app_id).join("db").join(normalized)
    }

    fn directory_usage(path: &Path) -> u64 {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(?path, %error, "Failed to read database usage");
                }
                return 0;
            }
        };

        entries
            .filter_map(|entry| {
                entry
                    .inspect_err(|error| tracing::warn!(?path, %error, "Failed to read database entry"))
                    .ok()
            })
            .map(|entry| {
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(error) => {
                        tracing::warn!(?path, %error, "Failed to inspect database entry");
                        return 0;
                    }
                };
                if file_type.is_file() {
                    entry.metadata().map(|metadata| metadata.len()).unwrap_or_else(|error| {
                        tracing::warn!(?path, %error, "Failed to read record size");
                        0
                    })
                } else if file_type.is_dir() {
                    Self::directory_usage(&entry.path())
                } else {
                    0
                }
            })
            .sum()
    }
}

#[async_trait::async_trait]
impl BackendDatabaseRepository for DatabaseRepository {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        let base_path = self.path_for(name, app_id);
        if let Err(error) = fs::create_dir_all(&base_path) {
            tracing::warn!(?base_path, %error, "Failed to open database");
        }
        Box::new(DiskDatabase { base_path })
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        let path = self.path_for(name, app_id);
        path.try_exists().unwrap_or_else(|error| {
            tracing::warn!(?path, %error, "Failed to check database existence");
            false
        })
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        let path = self.path_for(name, app_id);
        match fs::remove_dir_all(&path) {
            Ok(()) => true,
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(?path, %error, "Failed to delete database");
                }
                false
            }
        }
    }

    async fn usage(&self, app_id: &str) -> u64 {
        Self::directory_usage(&self.base_path.join(app_id).join("db"))
    }
}

struct DiskDatabase {
    base_path: PathBuf,
}

impl DiskDatabase {
    fn find_empty_record_id(&self) -> RecordId {
        // MIDP record IDs start at one; zero represents an I/O failure.
        for id in 1..=RecordId::MAX {
            let path = self.base_path.join(id.to_string());
            match path.try_exists() {
                Ok(false) => return id,
                Ok(true) => {}
                Err(error) => {
                    tracing::warn!(?path, %error, "Failed to find an unused record ID");
                    return 0;
                }
            }
        }
        tracing::warn!(path = ?self.base_path, "No unused record ID remains");
        0
    }
}

#[async_trait::async_trait]
impl Database for DiskDatabase {
    async fn next_id(&self) -> RecordId {
        self.find_empty_record_id()
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let id = self.find_empty_record_id();
        if id != 0 && self.set(id, data).await { id } else { 0 }
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        let path = self.base_path.join(id.to_string());
        fs::read(&path)
            .inspect_err(|error| {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(?path, %error, "Failed to read record");
                }
            })
            .ok()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let path = self.base_path.join(id.to_string());
        fs::write(&path, data)
            .inspect_err(|error| tracing::warn!(?path, %error, "Failed to write record"))
            .is_ok()
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        let path = self.base_path.join(id.to_string());
        fs::remove_file(&path)
            .inspect_err(|error| {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(?path, %error, "Failed to delete record");
                }
            })
            .is_ok()
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        let entries = match fs::read_dir(&self.base_path) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(path = ?self.base_path, %error, "Failed to list records");
                return Vec::new();
            }
        };
        entries
            .filter_map(|entry| {
                let entry = entry.inspect_err(|error| tracing::warn!(%error, "Failed to read record entry")).ok()?;
                if !entry
                    .file_type()
                    .inspect_err(|error| tracing::warn!(%error, "Failed to inspect record entry"))
                    .ok()?
                    .is_file()
                {
                    return None;
                }
                entry.file_name().to_str()?.parse().ok()
            })
            .collect()
    }
}
