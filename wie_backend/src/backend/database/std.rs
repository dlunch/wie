use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use super::RecordId;
pub struct DatabaseRepository {
    base_path: PathBuf,
}

impl DatabaseRepository {
    pub fn new(app_id: &str) -> Self {
        let base_dir = ProjectDirs::from("net", "dlunch", "wie").unwrap();

        let base_path = base_dir.data_dir().join(app_id);

        Self { base_path }
    }

    pub fn open(&mut self, name: &str) -> anyhow::Result<Database> {
        let path = self.get_path_for_database(name);

        Database::new(path)
    }

    pub fn delete(&mut self, name: &str) -> anyhow::Result<()> {
        let path = self.get_path_for_database(name);

        fs::remove_dir_all(path)?;

        Ok(())
    }

    fn get_path_for_database(&self, name: &str) -> PathBuf {
        self.base_path.join(name)
    }
}

pub struct Database {
    base_path: PathBuf,
}

impl Database {
    pub fn new(base_path: PathBuf) -> anyhow::Result<Self> {
        tracing::trace!("Opening database at {:?}", base_path);

        // TODO wasm?
        fs::create_dir_all(&base_path)?;

        Ok(Self { base_path })
    }

    pub fn add(&mut self, data: &[u8]) -> anyhow::Result<RecordId> {
        let id = self.find_empty_record_id()?;

        tracing::trace!("Adding record {} to database {:?}", id, &self.base_path);

        let path = self.get_path_for_record(id);
        fs::write(path, data)?;

        Ok(id)
    }

    pub fn get(&mut self, id: RecordId) -> anyhow::Result<Vec<u8>> {
        let path = self.get_path_for_record(id);

        tracing::trace!("Read record {} from database {:?}", id, &self.base_path);

        Ok(fs::read(path)?)
    }

    pub fn delete(&mut self, id: RecordId) -> anyhow::Result<()> {
        let path = self.get_path_for_record(id);

        tracing::trace!("Delete record {} from database {:?}", id, &self.base_path);

        fs::remove_file(path)?;

        Ok(())
    }

    pub fn set(&mut self, id: RecordId, data: &[u8]) -> anyhow::Result<()> {
        let path = self.get_path_for_record(id);

        tracing::trace!("Set record {} to database {:?}", id, &self.base_path);

        fs::write(path, data)?;

        Ok(())
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        tracing::trace!("Counting records in database {:?}", &self.base_path);

        Ok(fs::read_dir(&self.base_path)?.filter(|x| x.as_ref().unwrap().path().is_file()).count())
    }

    pub fn get_record_ids(&self) -> anyhow::Result<Vec<RecordId>> {
        Ok(fs::read_dir(&self.base_path)?
            .filter(|x| x.as_ref().unwrap().path().is_file())
            .map(|x| x.unwrap().file_name().to_str().unwrap().parse().unwrap())
            .collect())
    }

    fn get_path_for_record(&self, id: RecordId) -> PathBuf {
        self.base_path.join(id.to_string())
    }

    fn find_empty_record_id(&mut self) -> anyhow::Result<RecordId> {
        let mut record_id = 0;

        loop {
            let path = self.base_path.join(record_id.to_string());

            if !path.exists() {
                return Ok(record_id);
            }

            record_id += 1;
        }
    }
}
