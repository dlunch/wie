use std::{
    fs::{self, File},
    io::ErrorKind,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow, ensure};
use serde::{Deserialize, Serialize};
use tempfile::{NamedTempFile, tempdir_in};

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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub midi_volume: f32,
    pub pcm_volume: f32,
    pub help_dismissed: bool,
    pub welcome_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            midi_volume: 0.5,
            pcm_volume: 0.5,
            help_dismissed: false,
            welcome_seen: false,
        }
    }
}

pub struct Library {
    root: PathBuf,
}

impl Library {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn list(&self) -> Result<Vec<LibraryApp>> {
        let entries = match fs::read_dir(self.root.join("library/apps")) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut apps: Vec<LibraryApp> = Vec::new();
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                apps.push(serde_json::from_reader(File::open(entry.path().join("metadata.json"))?)?);
            }
        }
        apps.sort_by(|left, right| left.added_at.cmp(&right.added_at).then_with(|| left.id.cmp(&right.id)));
        Ok(apps)
    }

    pub fn import(&mut self, filename: &str, bytes: &[u8]) -> Result<()> {
        let metadata = extract_app_metadata(filename, bytes)?;
        ensure!(!self.list()?.iter().any(|app| app.id == metadata.id), "App is already imported");
        let app = LibraryApp {
            id: metadata.id,
            title: metadata.title,
            filename: filename.to_owned(),
            added_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().try_into()?,
            icon: metadata.icon,
        };
        let library_root = self.root.join("library");
        let apps = library_root.join("apps");
        fs::create_dir_all(&apps)?;
        // Stage outside the listed directory, then publish the complete entry in one rename.
        let staging = tempdir_in(library_root)?;
        fs::write(staging.path().join("archive"), bytes)?;
        serde_json::to_writer(File::create(staging.path().join("metadata.json"))?, &app)?;
        fs::rename(staging.path(), apps.join(&app.id))?;
        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        let app = self
            .list()?
            .into_iter()
            .find(|app| app.id == id)
            .ok_or_else(|| anyhow!("App is not imported"))?;
        fs::remove_dir_all(self.root.join("library/apps").join(app.id))?;
        Ok(())
    }

    pub fn read_archive(&self, id: &str) -> Result<(LibraryApp, Vec<u8>)> {
        let app = self
            .list()?
            .into_iter()
            .find(|app| app.id == id)
            .ok_or_else(|| anyhow!("App is not imported"))?;
        let bytes = fs::read(self.root.join("library/apps").join(&app.id).join("archive"))?;
        Ok((app, bytes))
    }

    pub fn read_settings(&self) -> Result<Settings> {
        match File::open(self.root.join("settings.json")) {
            Ok(file) => Ok(serde_json::from_reader(file)?),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Settings::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn write_settings(&mut self, settings: &Settings) -> Result<()> {
        ensure!(
            (0.0..=1.0).contains(&settings.midi_volume) && (0.0..=1.0).contains(&settings.pcm_volume),
            "Volume must be between 0 and 1"
        );
        fs::create_dir_all(&self.root)?;
        let temporary = NamedTempFile::new_in(&self.root)?;
        serde_json::to_writer(temporary.as_file(), settings)?;
        temporary.persist(self.root.join("settings.json"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Write},
    };

    use tempfile::tempdir;
    use wie_backend::{DatabaseRepository as _, Filesystem as _};
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::{database::DatabaseRepository, filesystem::DiskFilesystem};

    use super::{Library, Settings};

    fn archive(bytes: &[u8], descriptor: &str, id: &str) -> Vec<u8> {
        let mut files = wie_backend::extract_zip(bytes).unwrap();
        files
            .get_mut(descriptor)
            .unwrap()
            .extend_from_slice(format!("\nName:Test app\nAID:{id}\nPID:{id}\n").as_bytes());
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
        let mut library = Library::new(root.path().to_owned());
        let ktf = archive(include_bytes!("../../wie-ktf/tests/data/helloworld_ktf.zip"), "__adf__", "00000000");
        let lgt = archive(include_bytes!("../../wie-lgt/tests/data/helloworld_lgt.zip"), "app_info", "00000001");
        assert!(library.list().unwrap().is_empty());
        library.import("hello.zip", &ktf).unwrap();
        library.import("other.zip", &lgt).unwrap();
        let imported = library.list().unwrap();
        let [first, second] = imported.as_slice() else {
            panic!("Both apps should be imported");
        };
        assert_ne!(first.id, second.id);
        assert!(library.import("duplicate.zip", &ktf).is_err());

        tauri::async_runtime::block_on(async {
            let repository = DatabaseRepository::new(root.path().to_owned());
            let filesystem = DiskFilesystem::new(root.path().to_owned());
            for (id, data) in [(&first.id, b"record".as_slice()), (&second.id, b"other".as_slice())] {
                let mut database = repository.open("save", id).await;
                assert_eq!(database.add(data).await, 1);
                assert_eq!(filesystem.write(id, "save", 2, data).await, data.len());
            }
        });

        let mut library = Library::new(root.path().to_owned());
        let (metadata, bytes) = library.read_archive(&first.id).unwrap();
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
        tauri::async_runtime::block_on(async {
            let repository = DatabaseRepository::new(root.path().to_owned());
            let filesystem = DiskFilesystem::new(root.path().to_owned());
            for (id, data) in [(&first.id, b"record".as_slice()), (&second.id, b"other".as_slice())] {
                assert_eq!(repository.open("save", id).await.get(1).await.as_deref(), Some(data));
                assert_eq!(repository.usage(id).await, data.len() as u64);
                assert_eq!(filesystem.size(id, "save").await, Some(data.len() + 2));
                let mut bytes = vec![0; data.len() + 2];
                assert_eq!(filesystem.read(id, "save", 0, bytes.len(), &mut bytes).await, Some(bytes.len()));
                assert_eq!(&bytes[..2], &[0, 0]);
                assert_eq!(&bytes[2..], data);
            }
        });
        assert!(library.read_archive("../outside").is_err());
        assert!(library.delete("../outside").is_err());
        library.import("hello.zip", &ktf).unwrap();
        assert_eq!(library.list().unwrap().len(), 2);
    }

    #[test]
    fn settings_persist_and_failed_updates_preserve_previous_values() {
        let root = tempdir().unwrap();
        let mut library = Library::new(root.path().to_owned());
        let defaults = library.read_settings().unwrap();
        assert_eq!(defaults.midi_volume, 0.5);
        assert_eq!(defaults.pcm_volume, 0.5);
        assert!(!defaults.help_dismissed);
        assert!(!defaults.welcome_seen);

        let settings = Settings {
            midi_volume: 0.25,
            pcm_volume: 0.75,
            help_dismissed: true,
            welcome_seen: true,
        };
        library.write_settings(&settings).unwrap();
        library.write_settings(&settings).unwrap();
        let mut library = Library::new(root.path().to_owned());
        let loaded = library.read_settings().unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::json!({
                "midiVolume": 0.25,
                "pcmVolume": 0.75,
                "helpDismissed": true,
                "welcomeSeen": true,
            })
        );
        for volume in [f32::NAN, 1.1] {
            assert!(
                library
                    .write_settings(&Settings {
                        midi_volume: volume,
                        ..settings
                    })
                    .is_err()
            );
            assert_eq!(library.read_settings().unwrap().midi_volume, 0.25);
        }

        fs::write(root.path().join("settings.json"), b"invalid json").unwrap();
        assert!(library.read_settings().is_err());
    }
}
