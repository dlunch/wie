use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow, ensure};
use rusqlite::{Connection, Error as SqliteError, ErrorCode, OptionalExtension, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};

use wie::extract_app_metadata;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryApp {
    pub id: String,
    pub title: String,
    pub filename: String,
    pub added_at: u64,
    pub icon: Option<Vec<u8>>,
}

pub struct Library {
    connection: Connection,
}

impl Library {
    pub fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        let connection = Connection::open(root.join("library.sqlite"))?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS apps (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                filename TEXT NOT NULL,
                added_at INTEGER NOT NULL,
                icon BLOB,
                archive BLOB NOT NULL
            );",
        )?;
        Ok(Self { connection })
    }

    pub fn list(&self) -> Result<Vec<LibraryApp>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, title, filename, added_at, icon FROM apps ORDER BY added_at, id")?;
        Ok(statement
            .query_map([], |row| {
                Ok(LibraryApp {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    filename: row.get(2)?,
                    added_at: row.get(3)?,
                    icon: row.get(4)?,
                })
            })?
            .collect::<SqliteResult<_>>()?)
    }

    pub fn import(&mut self, filename: &str, bytes: &[u8]) -> Result<()> {
        let metadata = extract_app_metadata(filename, bytes)?;
        let added_at: u64 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().try_into()?;
        match self.connection.execute(
            "INSERT INTO apps (id, title, filename, added_at, icon, archive) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![metadata.id, metadata.title, filename, added_at, metadata.icon, bytes],
        ) {
            Ok(_) => {}
            Err(SqliteError::SqliteFailure(error, _)) if error.code == ErrorCode::ConstraintViolation => {
                return Err(anyhow!("App is already imported"));
            }
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        ensure!(
            self.connection.execute("DELETE FROM apps WHERE id = ?1", [id])? != 0,
            "App is not imported"
        );
        Ok(())
    }

    pub fn read_archive(&self, id: &str) -> Result<(LibraryApp, Vec<u8>)> {
        self.connection
            .query_row(
                "SELECT id, title, filename, added_at, icon, archive FROM apps WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        LibraryApp {
                            id: row.get(0)?,
                            title: row.get(1)?,
                            filename: row.get(2)?,
                            added_at: row.get(3)?,
                            icon: row.get(4)?,
                        },
                        row.get(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| anyhow!("App is not imported"))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Write},
    };

    use rusqlite::params;
    use tempfile::tempdir;
    use wie_backend::{DatabaseRepository as _, Filesystem as _};
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::{database::DatabaseRepository, filesystem::SqliteFilesystem, store};

    use super::Library;

    fn archive(bytes: &[u8], descriptor: &str, id: &str) -> Vec<u8> {
        let mut files = wie_backend::extract_zip(bytes).unwrap();
        files
            .get_mut(descriptor)
            .unwrap()
            .extend_from_slice(format!("\nName:Test app\nAID:{id}\nPID:PD{id}\n").as_bytes());
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in files {
            writer.start_file(name, SimpleFileOptions::default()).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn library_reopens_archives_and_preserves_guest_saves_on_delete() {
        let root = tempdir().unwrap();
        let ktf = archive(include_bytes!("../../wie-ktf/tests/data/helloworld_ktf.zip"), "__adf__", "00000000");
        let lgt = archive(include_bytes!("../../wie-lgt/tests/data/helloworld_lgt.zip"), "app_info", "00000001");
        let imported = {
            let mut library = Library::new(root.path().to_owned()).unwrap();
            assert!(library.list().unwrap().is_empty());
            library.import("hello.zip", &ktf).unwrap();
            library.import("other.zip", &lgt).unwrap();
            assert!(library.import("duplicate.zip", &ktf).is_err());
            library.list().unwrap()
        };
        let [first, second] = imported.as_slice() else {
            panic!("Both apps should be imported");
        };
        assert_eq!(first.id, "PD00000000");
        assert_eq!(second.id, "PD00000001");

        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            let repository = DatabaseRepository { store: store.clone() };
            let filesystem = SqliteFilesystem { store };
            for (id, aid, data) in [
                (&first.id, "00000000", b"record".as_slice()),
                (&second.id, "00000001", b"other".as_slice()),
            ] {
                let mut database = repository.open("save", id).await;
                assert!(database.set(1, data).await);
                assert_eq!(database.next_id().await, 2);
                assert_eq!(repository.usage(id).await, data.len() as u64);
                assert_eq!(filesystem.write(aid, "save", 2, data).await, data.len());
            }
        });

        {
            let mut library = Library::new(root.path().to_owned()).unwrap();
            let (metadata, bytes) = library.read_archive(&first.id).unwrap();
            assert_eq!(metadata.id, first.id);
            assert_eq!(metadata.filename, "hello.zip");
            assert_eq!(metadata.title, first.title);
            assert_eq!(metadata.added_at, first.added_at);
            assert_eq!(metadata.icon, first.icon);
            assert_eq!(bytes, ktf);
            library.delete(&first.id).unwrap();
            assert!(library.read_archive(&first.id).is_err());
            let apps = library.list().unwrap();
            assert_eq!(apps.len(), 1);
            assert_eq!(apps[0].id, second.id);
            assert!(library.read_archive("../outside").is_err());
            assert!(library.delete("../outside").is_err());
            library.import("hello.zip", &ktf).unwrap();
            assert_eq!(library.list().unwrap().len(), 2);
        }
        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            let repository = DatabaseRepository { store: store.clone() };
            let filesystem = SqliteFilesystem { store };
            for (id, aid, data) in [
                (&first.id, "00000000", b"record".as_slice()),
                (&second.id, "00000001", b"other".as_slice()),
            ] {
                assert_eq!(repository.database("save", id).get(1).await.as_deref(), Some(data));
                assert_eq!(repository.usage(id).await, data.len() as u64);
                assert_eq!(filesystem.size(aid, "save").await, Some(data.len() + 2));
                let mut bytes = vec![0; data.len() + 2];
                assert_eq!(filesystem.read(aid, "save", 0, bytes.len(), &mut bytes).await, Some(bytes.len()));
                assert_eq!(&bytes[..2], &[0, 0]);
                assert_eq!(&bytes[2..], data);
            }

            let mut database = repository.open("save", &first.id).await;
            assert_eq!(database.add(b"second").await, 2);
            assert!(database.set(1, b"updated").await);
            assert_eq!(repository.usage(&first.id).await, 13);
            assert_eq!(database.next_id().await, 3);
            assert!(database.delete(1).await);
            let mut reopened = repository.open("save", &first.id).await;
            assert_eq!(reopened.next_id().await, 3);
            assert_eq!(reopened.add(b"third").await, 3);
            assert_eq!(database.next_id().await, 4);
            assert!(reopened.delete(3).await);
            assert!(reopened.delete(2).await);

            for pid in ["pid/./app", "pid/app"] {
                for name in ["save:1", "save_1"] {
                    assert_eq!(repository.open(name, pid).await.add(format!("{pid}:{name}").as_bytes()).await, 1);
                }
            }
        });

        // All state owners above have closed, including handles to the now-empty store.
        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            let repository = DatabaseRepository { store: store.clone() };
            let filesystem = SqliteFilesystem { store: store.clone() };
            assert!(repository.exists("save", &first.id).await);
            let mut reopened = repository.open("save", &first.id).await;
            assert!(reopened.get_record_ids().await.is_empty());
            assert_eq!(reopened.next_id().await, 4);

            store
                .lock()
                .unwrap()
                .execute_batch(
                    "CREATE TEMP TRIGGER fail_record_insert BEFORE INSERT ON records
                     BEGIN SELECT RAISE(ABORT, 'injected record failure'); END;",
                )
                .unwrap();
            assert_eq!(reopened.add(b"failed").await, 0);
            assert_eq!(reopened.next_id().await, 4);
            assert!(reopened.get_record_ids().await.is_empty());
            assert_eq!(repository.usage(&first.id).await, 0);
            store.lock().unwrap().execute_batch("DROP TRIGGER fail_record_insert").unwrap();
            assert_eq!(reopened.add(b"fourth").await, 4);
            assert_eq!(reopened.get_record_ids().await, [4]);
            assert_eq!(repository.usage(&first.id).await, 6);

            store
                .lock()
                .unwrap()
                .execute(
                    "UPDATE record_stores SET next_id = ?1 WHERE pid = ?2 AND name = 'save'",
                    params![i32::MAX, first.id],
                )
                .unwrap();
            assert_eq!(reopened.next_id().await, i32::MAX as u32);
            assert_eq!(reopened.add(b"last").await, i32::MAX as u32);
            assert_eq!(reopened.get_record_ids().await, [4, i32::MAX as u32]);
            assert_eq!(reopened.next_id().await, 0);
            assert_eq!(reopened.add(b"exhausted").await, 0);
            assert!(reopened.delete(i32::MAX as u32).await);
            assert_eq!(reopened.next_id().await, 0);

            assert!(repository.delete("save", &first.id).await);
            assert_eq!(repository.database("save", &first.id).get(1).await, None);
            assert!(!repository.exists("save", &first.id).await);
            assert_eq!(repository.usage(&first.id).await, 0);
            let mut reset = repository.open("save", &first.id).await;
            assert!(reset.get_record_ids().await.is_empty());
            assert_eq!(reset.add(b"reset").await, 1);
            assert_eq!(reset.get_record_ids().await, [1]);
            assert_eq!(repository.usage(&first.id).await, 5);

            for pid in ["pid/./app", "pid/app"] {
                for name in ["save:1", "save_1"] {
                    assert_eq!(repository.open(name, pid).await.get(1).await.unwrap(), format!("{pid}:{name}").as_bytes());
                }
            }
            assert_eq!(repository.usage("pid/./app").await, 32);
            assert_eq!(repository.usage("pid/app").await, 28);
            assert!(repository.delete("save:1", "pid/./app").await);
            assert_eq!(
                repository.open("save_1", "pid/./app").await.get(1).await.as_deref(),
                Some(b"pid/./app:save_1".as_slice())
            );
            assert_eq!(
                repository.open("save:1", "pid/app").await.get(1).await.as_deref(),
                Some(b"pid/app:save:1".as_slice())
            );

            assert_eq!(filesystem.write("00000000", "save", 3, &[0xff, 0x00]).await, 2);
            let mut bytes = [0; 8];
            assert_eq!(filesystem.read("00000000", "save", 0, 8, &mut bytes).await, Some(8));
            assert_eq!(&bytes, b"\0\0r\xff\0ord");
            assert_eq!(filesystem.read("00000000", "save", 7, 8, &mut bytes).await, Some(1));
            assert_eq!(&bytes, b"d\0r\xff\0ord");
            assert_eq!(filesystem.read("00000000", "save", 8, 8, &mut bytes).await, Some(0));
            assert_eq!(&bytes, b"d\0r\xff\0ord");

            store
                .lock()
                .unwrap()
                .execute_batch(
                    "CREATE TEMP TRIGGER fail_file_growth AFTER UPDATE ON files
                     WHEN length(NEW.data) > length(OLD.data)
                     BEGIN SELECT RAISE(ABORT, 'injected file growth failure'); END;",
                )
                .unwrap();
            assert_eq!(filesystem.write("00000000", "save", 7, b"extension").await, 0);
            assert_eq!(filesystem.size("00000000", "save").await, Some(8));
            assert_eq!(filesystem.read("00000000", "save", 0, 8, &mut bytes).await, Some(8));
            assert_eq!(&bytes, b"\0\0r\xff\0ord");
            assert_eq!(filesystem.write("00000000", "failed", 2, b"new").await, 0);
            assert!(!filesystem.exists("00000000", "failed").await);
            store.lock().unwrap().execute_batch("DROP TRIGGER fail_file_growth").unwrap();

            filesystem.truncate("00000000", "save", 3).await;
            filesystem.truncate("00000000", "save", 6).await;
            assert_eq!(filesystem.read("00000000", "save", 0, 8, &mut bytes).await, Some(6));
            assert_eq!(&bytes[..6], b"\0\0r\0\0\0");
            assert_eq!(filesystem.write("00000000", "empty", 5, &[]).await, 0);
            assert_eq!(filesystem.size("00000000", "empty").await, Some(5));
            filesystem.truncate("00000000", "empty", 0).await;
            filesystem.truncate("00000000", "new", 0).await;
            assert!(filesystem.exists("00000000", "new").await);
            assert_eq!(filesystem.size("00000000", "new").await, Some(0));
            assert_eq!(filesystem.read("00000000", "missing", 0, 8, &mut bytes).await, None);
        });

        // Cloud backup excludes the library; restore only the closed state database.
        let restored = tempdir().unwrap();
        fs::copy(root.path().join("state.sqlite"), restored.path().join("state.sqlite")).unwrap();
        assert!(!restored.path().join("library.sqlite").exists());
        let mut library = Library::new(restored.path().to_owned()).unwrap();
        assert!(library.list().unwrap().is_empty());
        library.import("hello.zip", &ktf).unwrap();
        library.import("other.zip", &lgt).unwrap();
        assert_eq!(library.list().unwrap().len(), 2);
        assert_eq!(library.read_archive(&first.id).unwrap().1, ktf);
        assert_eq!(library.read_archive(&second.id).unwrap().1, lgt);
        tauri::async_runtime::block_on(async {
            let store = store::open(restored.path()).unwrap();
            let repository = DatabaseRepository { store: store.clone() };
            let filesystem = SqliteFilesystem { store };
            for (pid, aid, record, file) in [
                (&first.id, "00000000", b"reset".as_slice(), b"\0\0r\0\0\0".as_slice()),
                (&second.id, "00000001", b"other".as_slice(), b"\0\0other".as_slice()),
            ] {
                let database = repository.open("save", pid).await;
                assert_eq!(database.get(1).await.as_deref(), Some(record));
                assert_eq!(database.get_record_ids().await, [1]);
                assert_eq!(database.next_id().await, 2);
                assert_eq!(repository.usage(pid).await, record.len() as u64);
                assert_eq!(filesystem.size(aid, "save").await, Some(file.len()));
                let mut bytes = [0; 8];
                assert_eq!(filesystem.read(aid, "save", 0, bytes.len(), &mut bytes).await, Some(file.len()));
                assert_eq!(&bytes[..file.len()], file);
            }
            assert!(filesystem.exists("00000000", "empty").await);
            assert_eq!(filesystem.size("00000000", "empty").await, Some(0));
            assert_eq!(filesystem.read("00000000", "empty", 0, 1, &mut [1]).await, Some(0));
        });
    }
}
