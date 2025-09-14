use alloc::{boxed::Box, vec::Vec};

use crate::System;

pub type RecordId = u32;

#[async_trait::async_trait]
pub trait Database: Send {
    async fn next_id(&self) -> RecordId;
    async fn add(&mut self, data: &[u8]) -> RecordId;
    async fn get(&self, id: RecordId) -> Option<Vec<u8>>;
    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool;
    async fn delete(&mut self, id: RecordId) -> bool;

    async fn get_record_ids(&self) -> Vec<RecordId>;
}

#[async_trait::async_trait]
pub trait DatabaseRepository {
    async fn open(&self, system: &System, name: &str, app_id: &str) -> Box<dyn Database>;
    async fn exists(&self, system: &System, name: &str, app_id: &str) -> bool;
}
