use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow, ensure};
use rusqlite::{Connection, Error as SqliteError, ErrorCode, OptionalExtension, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};

use wie::{determine_file_type, extract_app_metadata};

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
        let file_type = determine_file_type(filename, bytes)?;
        let metadata = extract_app_metadata(filename, bytes, file_type)?;
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

    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use wie::{FileType, extract_app_metadata};
    use wie_backend::{DatabaseRepository as _, Filesystem as _};

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
    fn library_rejects_unsupported_ktf_apps() {
        let root = tempdir().unwrap();
        let mut library = Library::new(root.path().to_owned()).unwrap();
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("META-INF/MANIFEST.MF", SimpleFileOptions::default()).unwrap();
        writer
            .write_all(b"MIDlet-Name: Unsupported app\nMIDlet-1: Unsupported app, , example.Main\n")
            .unwrap();
        writer.start_file("client.bin40", SimpleFileOptions::default()).unwrap();
        writer.write_all(&[0x28, 0, 0, 0, 0x1c, 0x1b, 0, 0, 0, 0, 0, 0]).unwrap();
        let jar = writer.finish().unwrap().into_inner();

        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("__adf__", SimpleFileOptions::default()).unwrap();
        writer
            .write_all(b"Name:Unsupported app\nAID:test\nPID:PDtest\nMClass:example.Main\n")
            .unwrap();
        writer.start_file("test.jar", SimpleFileOptions::default()).unwrap();
        writer.write_all(&jar).unwrap();
        let archive = writer.finish().unwrap().into_inner();

        for (filename, bytes) in [("test.jar", jar), ("test.zip", archive)] {
            let metadata = extract_app_metadata(filename, &bytes, FileType::Ktf).unwrap();
            assert_eq!(metadata.title, "Unsupported app");
            let error = library.import(filename, &bytes).unwrap_err();
            assert_eq!(error.to_string(), "Fatal error: Unsupported KTF client.bin format");
            assert!(library.list().unwrap().is_empty());
        }
    }

    #[test]
    fn library_import_uses_midlet_display_name() {
        let root = tempdir().unwrap();
        let mut library = Library::new(root.path().to_owned()).unwrap();
        for (filename, midlet_name, expected) in [("named.jar", " App title ", "App title"), ("other.jar", "Other app", "Other app")] {
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            writer.start_file("META-INF/MANIFEST.MF", SimpleFileOptions::default()).unwrap();
            write!(writer, "MIDlet-Name: Suite title\nMIDlet-1: {midlet_name}, /icon.png, example.Main\n").unwrap();
            writer.start_file("icon.png", SimpleFileOptions::default()).unwrap();
            writer.write_all(b"icon").unwrap();
            let jar = writer.finish().unwrap().into_inner();
            library.import(filename, &jar).unwrap();
            let (metadata, _) = library.read_archive(expected).unwrap();
            assert_eq!(metadata.title, expected);
            assert_eq!(metadata.id, expected);
            assert_eq!(metadata.icon.as_deref(), Some(b"icon".as_slice()));
            assert_eq!(library.import("renamed.jar", &jar).unwrap_err().to_string(), "App is already imported");
        }
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

        let saves = [(first.id.as_str(), b"record".as_slice()), (second.id.as_str(), b"other".as_slice())];
        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            let repository = DatabaseRepository { store: store.clone() };
            let filesystem = SqliteFilesystem { store };
            for (id, data) in saves {
                repository.create("save", id).await;
                assert!(repository.open("save", id).await.set(1, data).await);
                assert_eq!(filesystem.write(id, "save", 0, data).await, data.len());
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
            for (pid, data) in saves {
                assert_eq!(repository.open("save", pid).await.get(1).await.as_deref(), Some(data));
                let mut bytes = [0; 8];
                assert_eq!(filesystem.read(pid, "save", 0, bytes.len(), &mut bytes).await, Some(data.len()));
                assert_eq!(&bytes[..data.len()], data);
            }
        });
    }
}
