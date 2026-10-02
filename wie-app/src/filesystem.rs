use anyhow::{Context, Result};
use rusqlite::{MAIN_DB, OptionalExtension, params};

use wie_backend::Filesystem;

use crate::store::Store;

pub(crate) struct SqliteFilesystem {
    pub(crate) store: Store,
}

#[async_trait::async_trait]
impl Filesystem for SqliteFilesystem {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        self.store
            .lock()
            .unwrap()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM files WHERE aid = ?1 AND path = ?2)",
                params![aid, path],
                |row| row.get(0),
            )
            .unwrap_or_else(|error| {
                tracing::warn!(aid, path, %error, "Failed to check file existence");
                false
            })
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        self.store
            .lock()
            .unwrap()
            .query_row("SELECT length(data) FROM files WHERE aid = ?1 AND path = ?2", params![aid, path], |row| {
                row.get(0)
            })
            .optional()
            .unwrap_or_else(|error| {
                tracing::warn!(aid, path, %error, "Failed to read file size");
                None
            })
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let result = (|| -> Result<Option<usize>> {
            let connection = self.store.lock().unwrap();
            let rowid: Option<i64> = connection
                .query_row("SELECT rowid FROM files WHERE aid = ?1 AND path = ?2", params![aid, path], |row| {
                    row.get(0)
                })
                .optional()?;
            let Some(rowid) = rowid else {
                return Ok(None);
            };
            let blob = connection.blob_open(MAIN_DB, "files", "data", rowid, true)?;
            Ok(Some(blob.read_at(&mut buf[..count], offset)?))
        })();
        result.unwrap_or_else(|error| {
            tracing::warn!(aid, path, %error, "Failed to read file");
            None
        })
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let result = (|| -> Result<()> {
            let length = offset.checked_add(data.len()).context("File write offset overflow")?;
            let mut connection = self.store.lock().unwrap();
            let transaction = connection.transaction()?;
            transaction.execute(
                "INSERT INTO files (aid, path, data) VALUES (?1, ?2, X'') ON CONFLICT DO NOTHING",
                params![aid, path],
            )?;
            transaction.execute(
                "UPDATE files SET data = CAST(data || zeroblob(?3 - length(data)) AS BLOB)
                 WHERE aid = ?1 AND path = ?2 AND length(data) < ?3",
                params![aid, path, length],
            )?;
            let rowid = transaction.query_row("SELECT rowid FROM files WHERE aid = ?1 AND path = ?2", params![aid, path], |row| {
                row.get(0)
            })?;
            let mut blob = transaction.blob_open(MAIN_DB, "files", "data", rowid, false)?;
            blob.write_at(data, offset)?;
            blob.close()?;
            transaction.commit()?;
            Ok(())
        })();
        match result {
            Ok(()) => data.len(),
            Err(error) => {
                tracing::warn!(aid, path, %error, "Failed to write file");
                0
            }
        }
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        let result = self.store.lock().unwrap().execute(
            "INSERT INTO files (aid, path, data) VALUES (?1, ?2, zeroblob(?3))
             ON CONFLICT (aid, path) DO UPDATE SET data =
                 CASE WHEN ?3 < length(data) THEN substr(data, 1, ?3)
                      ELSE CAST(data || zeroblob(?3 - length(data)) AS BLOB) END
             WHERE length(data) != ?3",
            params![aid, path, len],
        );
        if let Err(error) = result {
            tracing::warn!(aid, path, %error, "Failed to truncate file");
        }
    }
}
