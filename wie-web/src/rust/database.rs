use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::RecordId;

use crate::indexed_db_store::{Store, StoreKey};

pub struct DatabaseRepository {}

impl DatabaseRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl wie_backend::DatabaseRepository for DatabaseRepository {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn wie_backend::Database> {
        let db_name = format!("wie_{app_id}");
        let store = Store::open(&db_name, &db_name).await;
        Box::new(Database {
            store,
            key_prefix: name.to_string(),
        })
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        let db_name = format!("wie_{app_id}");
        let store = Store::open(&db_name, &db_name).await;
        store.get_all_keys().await.iter().any(|key| key.starts_with(name))
    }

    async fn delete(&self, _name: &str, _app_id: &str) -> bool {
        true // TODO
    }

    async fn usage(&self, app_id: &str) -> u64 {
        let db_name = format!("wie_{app_id}");
        let store = Store::open(&db_name, &db_name).await;
        let mut usage = 0;

        for key in store.get_all_keys().await {
            if let Some(data) = store.get(StoreKey::String(key)).await {
                usage += data.len() as u64;
            }
        }

        usage
    }
}

pub struct Database {
    store: Store,
    key_prefix: String,
}

impl Database {
    fn record_key(&self, id: RecordId) -> StoreKey {
        StoreKey::String(format!("{}{}", self.key_prefix, id))
    }
}

#[async_trait::async_trait]
impl wie_backend::Database for Database {
    async fn add(&mut self, data: &[u8]) -> RecordId {
        let id = self.next_id().await;
        self.set(id, data).await;

        id
    }

    async fn next_id(&self) -> RecordId {
        let ids = self.get_record_ids().await;

        ids.iter().max().map_or(1, |&id| id + 1)
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        self.store.get(self.record_key(id)).await.map(|a| a.to_vec())
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        self.store.set(self.record_key(id), data).await;
        true
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.store.delete(self.record_key(id)).await;
        true
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        self.store
            .get_all_keys()
            .await
            .iter()
            .filter_map(|key| key.strip_prefix(self.key_prefix.as_str()).and_then(|tail| tail.parse::<RecordId>().ok()))
            .collect()
    }
}
