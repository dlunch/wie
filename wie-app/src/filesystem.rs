use anyhow::{Context, Result};
use rusqlite::{MAIN_DB, OptionalExtension, params};

use wie_backend::Filesystem;

use crate::store::Store;

pub(crate) struct SqliteFilesystem {
    pub(crate) store: Store,
}

#[async_trait::async_trait]
impl Filesystem for SqliteFilesystem {
    async fn remove(&self, pid: &str, path: &str) -> bool {
        self.store
            .lock()
            .unwrap()
            .execute("DELETE FROM files WHERE pid = ?1 AND path = ?2", params![pid, path])
            .map(|count| count != 0)
            .unwrap_or_else(|error| {
                tracing::warn!(pid, path, %error, "Failed to remove file");
                false
            })
    }

    async fn usage(&self, pid: &str) -> u64 {
        self.store
            .lock()
            .unwrap()
            .query_row("SELECT COALESCE(SUM(length(data)), 0) FROM files WHERE pid = ?1", [pid], |row| row.get(0))
            .unwrap_or_else(|error| {
                tracing::warn!(pid, %error, "Failed to read filesystem usage");
                0
            })
    }

    async fn exists(&self, pid: &str, path: &str) -> bool {
        self.store
            .lock()
            .unwrap()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM files WHERE pid = ?1 AND path = ?2)",
                params![pid, path],
                |row| row.get(0),
            )
            .unwrap_or_else(|error| {
                tracing::warn!(pid, path, %error, "Failed to check file existence");
                false
            })
    }

    async fn size(&self, pid: &str, path: &str) -> Option<usize> {
        self.store
            .lock()
            .unwrap()
            .query_row("SELECT length(data) FROM files WHERE pid = ?1 AND path = ?2", params![pid, path], |row| {
                row.get(0)
            })
            .optional()
            .unwrap_or_else(|error| {
                tracing::warn!(pid, path, %error, "Failed to read file size");
                None
            })
    }

    async fn read(&self, pid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let result = (|| -> Result<Option<usize>> {
            let connection = self.store.lock().unwrap();
            let rowid: Option<i64> = connection
                .query_row("SELECT rowid FROM files WHERE pid = ?1 AND path = ?2", params![pid, path], |row| {
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
            tracing::warn!(pid, path, %error, "Failed to read file");
            None
        })
    }

    async fn write(&self, pid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let result = (|| -> Result<()> {
            let length = offset.checked_add(data.len()).context("File write offset overflow")?;
            let mut connection = self.store.lock().unwrap();
            let transaction = connection.transaction()?;
            transaction.execute(
                "INSERT INTO files (pid, path, data) VALUES (?1, ?2, X'') ON CONFLICT DO NOTHING",
                params![pid, path],
            )?;
            transaction.execute(
                "UPDATE files SET data = CAST(data || zeroblob(?3 - length(data)) AS BLOB)
                 WHERE pid = ?1 AND path = ?2 AND length(data) < ?3",
                params![pid, path, length],
            )?;
            let rowid = transaction.query_row("SELECT rowid FROM files WHERE pid = ?1 AND path = ?2", params![pid, path], |row| {
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
                tracing::warn!(pid, path, %error, "Failed to write file");
                0
            }
        }
    }

    async fn truncate(&self, pid: &str, path: &str, len: usize) {
        let result = self.store.lock().unwrap().execute(
            "INSERT INTO files (pid, path, data) VALUES (?1, ?2, zeroblob(?3))
             ON CONFLICT (pid, path) DO UPDATE SET data =
                 CASE WHEN ?3 < length(data) THEN substr(data, 1, ?3)
                      ELSE CAST(data || zeroblob(?3 - length(data)) AS BLOB) END
             WHERE length(data) != ?3",
            params![pid, path, len],
        );
        if let Err(error) = result {
            tracing::warn!(pid, path, %error, "Failed to truncate file");
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use wie_backend::Filesystem;

    use super::SqliteFilesystem;
    use crate::store;

    #[test]
    fn sparse_writes_and_reads_at_offsets() {
        let root = tempdir().unwrap();
        tauri::async_runtime::block_on(async {
            let fs = SqliteFilesystem {
                store: store::open(root.path()).unwrap(),
            };
            assert_eq!(fs.write("app", "save", 2, b"record").await, 6);
            assert_eq!(fs.write("app", "save", 3, &[0xff, 0]).await, 2);
            let mut bytes = [0; 8];
            assert_eq!(fs.read("app", "save", 0, 8, &mut bytes).await, Some(8));
            assert_eq!(&bytes, b"\0\0r\xff\0ord");
            assert_eq!(fs.read("app", "save", 7, 8, &mut bytes).await, Some(1));
            assert_eq!(&bytes, b"d\0r\xff\0ord");
            assert_eq!(fs.read("app", "save", 8, 8, &mut bytes).await, Some(0));
            assert_eq!(&bytes, b"d\0r\xff\0ord");
            assert_eq!(fs.read("app", "missing", 0, 8, &mut bytes).await, None);
        });
    }

    #[test]
    fn truncation_and_empty_writes_set_file_length() {
        let root = tempdir().unwrap();
        tauri::async_runtime::block_on(async {
            let fs = SqliteFilesystem {
                store: store::open(root.path()).unwrap(),
            };
            fs.write("app", "save", 0, b"record").await;
            fs.truncate("app", "save", 3).await;
            fs.truncate("app", "save", 6).await;
            let mut bytes = [0; 8];
            assert_eq!(fs.read("app", "save", 0, 8, &mut bytes).await, Some(6));
            assert_eq!(&bytes[..6], b"rec\0\0\0");
            assert_eq!(fs.write("app", "empty", 5, &[]).await, 0);
            assert_eq!(fs.size("app", "empty").await, Some(5));
            fs.truncate("app", "empty", 0).await;
            assert_eq!(fs.read("app", "empty", 0, 1, &mut bytes).await, Some(0));
            fs.truncate("app", "new", 0).await;
            assert!(fs.exists("app", "new").await);
            assert_eq!(fs.size("app", "new").await, Some(0));
        });
    }
}
