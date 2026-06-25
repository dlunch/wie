use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use wie_backend::RecordId;

pub struct DatabaseRepository {
    base_path: PathBuf,
}

impl DatabaseRepository {
    pub fn new() -> Self {
        let base_dir = ProjectDirs::from("net", "dlunch", "wie").unwrap();

        let base_path = base_dir.data_dir().to_owned();

        Self { base_path }
    }

    fn get_path_for_database(&self, name: &str, app_id: &str) -> PathBuf {
        let sanitized_app_id: String = app_id.chars().filter(|c| !matches!(c, '/' | '\\' | '\0')).collect();
        let app_id = if sanitized_app_id.is_empty() || sanitized_app_id == "." || sanitized_app_id == ".." {
            "_"
        } else {
            &sanitized_app_id
        };

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::DatabaseRepository;

    #[test]
    fn database_path_includes_db_segment() {
        let repo = DatabaseRepository {
            base_path: PathBuf::from("/tmp/wie_test"),
        };
        let path = repo.get_path_for_database("records", "game123");
        assert_eq!(path, PathBuf::from("/tmp/wie_test/game123/db/records"));
    }

    #[test]
    fn database_path_strips_guest_leading_slash() {
        let repo = DatabaseRepository {
            base_path: PathBuf::from("/tmp/wie_test"),
        };
        let path = repo.get_path_for_database("/save0.dat", "PD140106");
        assert_eq!(path, PathBuf::from("/tmp/wie_test/PD140106/db/save0.dat"));
    }

    #[test]
    fn database_path_does_not_escape_app_scope() {
        let repo = DatabaseRepository {
            base_path: PathBuf::from("/tmp/wie_test"),
        };
        let path = repo.get_path_for_database("/../save0.dat", "PD140106");
        assert!(path.starts_with(PathBuf::from("/tmp/wie_test/PD140106/db")));
    }
}
