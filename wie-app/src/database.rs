use rusqlite::{OptionalExtension, Result as SqliteResult, params};

use wie_backend::{Database, DatabaseRepository as BackendDatabaseRepository, RecordId};

use crate::store::Store;

pub(crate) struct DatabaseRepository {
    pub(crate) store: Store,
}

impl DatabaseRepository {
    // Access an existing store without recreating it during reads or deletes.
    pub(crate) fn database(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        Box::new(SqliteDatabase {
            store: self.store.clone(),
            pid: app_id.to_owned(),
            name: name.to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl BackendDatabaseRepository for DatabaseRepository {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        if let Err(error) = self.store.lock().unwrap().execute(
            "INSERT INTO record_stores (pid, name) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            params![app_id, name],
        ) {
            tracing::warn!(app_id, name, %error, "Failed to open database");
        }
        self.database(name, app_id)
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        self.store
            .lock()
            .unwrap()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM record_stores WHERE pid = ?1 AND name = ?2)",
                params![app_id, name],
                |row| row.get(0),
            )
            .unwrap_or_else(|error| {
                tracing::warn!(app_id, name, %error, "Failed to check database existence");
                false
            })
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        self.store
            .lock()
            .unwrap()
            .execute("DELETE FROM record_stores WHERE pid = ?1 AND name = ?2", params![app_id, name])
            .map(|count| count != 0)
            .unwrap_or_else(|error| {
                tracing::warn!(app_id, name, %error, "Failed to delete database");
                false
            })
    }

    async fn usage(&self, app_id: &str) -> u64 {
        self.store
            .lock()
            .unwrap()
            .query_row("SELECT COALESCE(SUM(length(data)), 0) FROM records WHERE pid = ?1", [app_id], |row| {
                row.get(0)
            })
            .unwrap_or_else(|error| {
                tracing::warn!(app_id, %error, "Failed to read database usage");
                0
            })
    }
}

struct SqliteDatabase {
    store: Store,
    pid: String,
    name: String,
}

#[async_trait::async_trait]
impl Database for SqliteDatabase {
    async fn next_id(&self) -> RecordId {
        self.store
            .lock()
            .unwrap()
            .query_row(
                "SELECT next_id FROM record_stores WHERE pid = ?1 AND name = ?2 AND next_id <= ?3",
                params![self.pid, self.name, i32::MAX],
                |row| row.get(0),
            )
            .unwrap_or_else(|error| {
                tracing::warn!(pid = self.pid, name = self.name, %error, "Failed to read next record ID");
                0
            })
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let result = (|| -> SqliteResult<RecordId> {
            let mut connection = self.store.lock().unwrap();
            let transaction = connection.transaction()?;
            let id = transaction.query_row(
                "UPDATE record_stores SET next_id = next_id + 1
                 WHERE pid = ?1 AND name = ?2 AND next_id <= ?3 RETURNING next_id - 1",
                params![self.pid, self.name, i32::MAX],
                |row| row.get::<_, RecordId>(0),
            )?;
            transaction.execute(
                "INSERT INTO records (pid, name, id, data) VALUES (?1, ?2, ?3, ?4)",
                params![self.pid, self.name, id, data],
            )?;
            transaction.commit()?;
            Ok(id)
        })();
        result.unwrap_or_else(|error| {
            tracing::warn!(pid = self.pid, name = self.name, %error, "Failed to add record");
            0
        })
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        self.store
            .lock()
            .unwrap()
            .query_row(
                "SELECT data FROM records WHERE pid = ?1 AND name = ?2 AND id = ?3",
                params![self.pid, self.name, id],
                |row| row.get(0),
            )
            .optional()
            .unwrap_or_else(|error| {
                tracing::warn!(pid = self.pid, name = self.name, id, %error, "Failed to read record");
                None
            })
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let result = (|| -> SqliteResult<()> {
            let mut connection = self.store.lock().unwrap();
            let transaction = connection.transaction()?;
            transaction.execute(
                "INSERT INTO records (pid, name, id, data) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (pid, name, id) DO UPDATE SET data = excluded.data",
                params![self.pid, self.name, id, data],
            )?;
            transaction.execute(
                "UPDATE record_stores SET next_id = MAX(next_id, ?3 + 1) WHERE pid = ?1 AND name = ?2",
                params![self.pid, self.name, id],
            )?;
            transaction.commit()
        })();
        result
            .inspect_err(|error| tracing::warn!(pid = self.pid, name = self.name, id, %error, "Failed to write record"))
            .is_ok()
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.store
            .lock()
            .unwrap()
            .execute(
                "DELETE FROM records WHERE pid = ?1 AND name = ?2 AND id = ?3",
                params![self.pid, self.name, id],
            )
            .map(|count| count != 0)
            .unwrap_or_else(|error| {
                tracing::warn!(pid = self.pid, name = self.name, id, %error, "Failed to delete record");
                false
            })
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        let result = (|| -> SqliteResult<Vec<RecordId>> {
            let connection = self.store.lock().unwrap();
            let mut statement = connection.prepare("SELECT id FROM records WHERE pid = ?1 AND name = ?2 ORDER BY id")?;
            statement.query_map(params![self.pid, self.name], |row| row.get(0))?.collect()
        })();
        result.unwrap_or_else(|error| {
            tracing::warn!(pid = self.pid, name = self.name, %error, "Failed to list records");
            Vec::new()
        })
    }
}
