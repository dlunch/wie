#![allow(dead_code)]

use super::RecordId;

pub struct DatabaseRepository {
    app_id: String,
}

impl DatabaseRepository {
    pub fn new(app_id: &str) -> Self {
        Self { app_id: app_id.to_owned() }
    }

    pub fn open(&mut self, name: &str) -> anyhow::Result<Database> {
        Database::new(name)
    }

    pub fn delete(&mut self, _name: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct Database {
    name: String,
}

impl Database {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        Ok(Self { name: name.to_owned() })
    }

    pub fn add(&mut self, _data: &[u8]) -> anyhow::Result<RecordId> {
        Ok(0)
    }

    pub fn get(&self, _id: RecordId) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    pub fn delete(&mut self, _id: RecordId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn set(&mut self, _id: RecordId, _data: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        Ok(0)
    }

    pub fn get_record_ids(&self) -> anyhow::Result<Vec<RecordId>> {
        Ok(Vec::new())
    }
}
