use std::{
    fs,
    path::{Path, PathBuf},
};

use wie_backend::RecordId;

pub struct DatabaseRepository {
    base_path: PathBuf,
}

impl DatabaseRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn get_path_for_database(&self, name: &str, app_id: &str) -> PathBuf {
        let name: String = name.chars().map(|c| if matches!(c, '\\' | '\0') { '_' } else { c }).collect();
        let mut normalized_name = PathBuf::new();
        for segment in name.trim_start_matches('/').split('/') {
            match segment {
                "" | "." => {}
                ".." => normalized_name.push("_"),
                segment => normalized_name.push(segment),
            }
        }
        if normalized_name.as_os_str().is_empty() {
            normalized_name.push("_");
        }

        self.base_path.join(app_id).join("db").join(normalized_name)
    }

    fn directory_usage(path: &Path) -> u64 {
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };

        entries
            .filter_map(Result::ok)
            .map(|entry| {
                let Ok(file_type) = entry.file_type() else {
                    return 0;
                };
                if file_type.is_file() {
                    entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
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
impl wie_backend::DatabaseRepository for DatabaseRepository {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn wie_backend::Database> {
        let path = self.get_path_for_database(name, app_id);

        Box::new(Database::new(path).unwrap())
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        let path = self.get_path_for_database(name, app_id);

        path.exists()
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        let path = self.get_path_for_database(name, app_id);

        tracing::trace!("Delete database at {path:?}");

        match fs::remove_dir_all(path) {
            Ok(()) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => {
                tracing::warn!("Failed to delete database: {e}");
                false
            }
        }
    }

    async fn usage(&self, app_id: &str) -> u64 {
        Self::directory_usage(&self.base_path.join(app_id).join("db"))
    }
}

pub struct Database {
    base_path: PathBuf,
}

impl Database {
    pub fn new(base_path: PathBuf) -> anyhow::Result<Self> {
        tracing::trace!("Opening database at {base_path:?}");

        fs::create_dir_all(&base_path)?;

        Ok(Self { base_path })
    }

    fn find_empty_record_id(&self) -> RecordId {
        let mut record_id = 1; // XXX midp requires first record to be 1

        loop {
            let path = self.base_path.join(record_id.to_string());

            if !path.exists() {
                return record_id;
            }

            record_id += 1;
        }
    }
    fn get_path_for_record(&self, id: RecordId) -> PathBuf {
        self.base_path.join(id.to_string())
    }
}

#[async_trait::async_trait]
impl wie_backend::Database for Database {
    async fn next_id(&self) -> RecordId {
        self.find_empty_record_id()
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let id = self.find_empty_record_id();

        tracing::trace!("Adding record {id} to database {:?}", &self.base_path);

        let path = self.get_path_for_record(id);
        fs::write(path, data).unwrap();

        id
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        let path = self.get_path_for_record(id);

        tracing::trace!("Read record {id} from database {:?}", &self.base_path);

        fs::read(path).ok()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let path = self.get_path_for_record(id);

        tracing::trace!("Set record {id} to database {:?}", &self.base_path);

        fs::write(path, data).is_ok()
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        let path = self.get_path_for_record(id);

        tracing::trace!("Delete record {id} from database {:?}", &self.base_path);

        fs::remove_file(path).is_ok()
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        fs::read_dir(&self.base_path)
            .unwrap()
            .filter(|x| x.as_ref().unwrap().path().is_file())
            .map(|x| x.unwrap().file_name().to_str().unwrap().parse().unwrap())
            .collect()
    }
}
