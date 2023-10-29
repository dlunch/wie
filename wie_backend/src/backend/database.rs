#![allow(dead_code)]
#![allow(unused_variables)]

pub type RecordId = u32;

pub struct DatabaseRepository {}

impl DatabaseRepository {
    pub fn new(app_id: &str) -> Self {
        Self {}
    }

    pub fn open(&mut self, name: &str) -> Database<'_> {
        unimplemented!()
    }

    pub fn delete(&mut self, name: &str) {
        unimplemented!()
    }
}

pub struct Database<'a> {
    repository: &'a DatabaseRepository,
}

impl<'a> Database<'a> {
    pub fn add(&mut self, data: &[u8]) -> anyhow::Result<RecordId> {
        unimplemented!()
    }

    pub fn get(&mut self, id: RecordId) -> Vec<u8> {
        unimplemented!()
    }

    pub fn delete(&mut self, id: RecordId) {
        unimplemented!()
    }

    pub fn set(&mut self, id: RecordId, data: &[u8]) {
        unimplemented!()
    }
}
