#![allow(unused_variables)]
use super::RecordId;

pub struct DatabaseRepository {}

impl DatabaseRepository {
    pub fn new(app_id: &str) -> Self {
        unimplemented!()
    }

    pub fn open(&mut self, name: &str) -> anyhow::Result<Database> {
        unimplemented!()
    }

    pub fn delete(&mut self, name: &str) -> anyhow::Result<()> {
        unimplemented!()
    }
}

pub struct Database {}

impl Database {
    pub fn new() -> anyhow::Result<Self> {
        unimplemented!()
    }

    pub fn add(&mut self, data: &[u8]) -> anyhow::Result<RecordId> {
        unimplemented!()
    }

    pub fn get(&mut self, id: RecordId) -> anyhow::Result<Vec<u8>> {
        unimplemented!()
    }

    pub fn delete(&mut self, id: RecordId) -> anyhow::Result<()> {
        unimplemented!()
    }

    pub fn set(&mut self, id: RecordId, data: &[u8]) -> anyhow::Result<()> {
        unimplemented!()
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        unimplemented!()
    }
}
