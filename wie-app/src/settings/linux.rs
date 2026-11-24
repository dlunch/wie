use std::{fs, path::Path};

use anyhow::{Result, ensure};
use gtk::glib::{FileError, FileSetContentsFlags, KeyFile, KeyFileFlags, file_set_contents_full};

use super::Settings;

pub(super) fn read(root: &Path) -> Result<Settings> {
    let file = KeyFile::new();
    if let Err(error) = file.load_from_file(root.join("settings.ini"), KeyFileFlags::NONE) {
        if error.matches(FileError::Noent) {
            return Ok(Settings::default());
        }
        return Err(error.into());
    }
    let mut settings = Settings::default();
    if file.has_group("Settings") {
        for (key, volume) in [("midiVolume", &mut settings.midi_volume), ("pcmVolume", &mut settings.pcm_volume)] {
            if file.has_key("Settings", key)? {
                let value = file.double("Settings", key)?;
                ensure!((0.0..=1.0).contains(&value), "Volume must be between 0 and 1");
                *volume = value as f32;
            }
        }
        for (key, flag) in [
            ("helpDismissed", &mut settings.help_dismissed),
            ("welcomeSeen", &mut settings.welcome_seen),
        ] {
            if file.has_key("Settings", key)? {
                *flag = file.boolean("Settings", key)?;
            }
        }
    }
    Ok(settings)
}

pub(super) fn write(root: &Path, settings: &Settings) -> Result<()> {
    settings.validate()?;
    let file = KeyFile::new();
    file.set_double("Settings", "midiVolume", settings.midi_volume.into());
    file.set_double("Settings", "pcmVolume", settings.pcm_volume.into());
    file.set_boolean("Settings", "helpDismissed", settings.help_dismissed);
    file.set_boolean("Settings", "welcomeSeen", settings.welcome_seen);
    fs::create_dir_all(root)?;
    file_set_contents_full(
        root.join("settings.ini"),
        file.to_data().as_bytes(),
        FileSetContentsFlags::CONSISTENT | FileSetContentsFlags::DURABLE,
        0o600,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{Settings, read, write};

    #[test]
    fn settings_persist_and_failed_updates_preserve_previous_values() {
        let temporary = tempdir().unwrap();
        let root = temporary.path().join("config");
        let defaults = read(&root).unwrap();
        assert_eq!(defaults.midi_volume, 0.5);
        assert_eq!(defaults.pcm_volume, 0.5);
        assert!(!defaults.help_dismissed);
        assert!(!defaults.welcome_seen);
        assert!(!root.exists());

        let settings = Settings {
            midi_volume: 0.25,
            pcm_volume: 0.75,
            help_dismissed: true,
            welcome_seen: true,
        };
        write(&root, &settings).unwrap();
        write(&root, &settings).unwrap();
        assert_eq!(
            serde_json::to_value(read(&root).unwrap()).unwrap(),
            serde_json::json!({
                "midiVolume": 0.25,
                "pcmVolume": 0.75,
                "helpDismissed": true,
                "welcomeSeen": true,
            })
        );
        let path = root.join("settings.ini");
        let saved = fs::read(&path).unwrap();
        for volume in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            for invalid in [
                Settings {
                    midi_volume: volume,
                    ..settings
                },
                Settings {
                    pcm_volume: volume,
                    ..settings
                },
            ] {
                assert!(write(&root, &invalid).is_err());
                assert_eq!(fs::read(&path).unwrap(), saved);
            }
        }

        for contents in [
            "invalid ini",
            "[Settings]\nmidiVolume=invalid\n",
            "[Settings]\npcmVolume=nan\n",
            "[Settings]\npcmVolume=1.00000001\n",
            "[Settings]\nhelpDismissed=invalid\n",
        ] {
            fs::write(&path, contents).unwrap();
            assert!(read(&root).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), contents);
        }
        fs::write(&path, "[Settings]\nwelcomeSeen=true\n").unwrap();
        let partial = read(&root).unwrap();
        assert_eq!(partial.midi_volume, 0.5);
        assert_eq!(partial.pcm_volume, 0.5);
        assert!(!partial.help_dismissed);
        assert!(partial.welcome_seen);

        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep"), b"existing data").unwrap();
        assert!(read(&root).is_err());
        assert!(write(&root, &settings).is_err());
        assert_eq!(fs::read(path.join("keep")).unwrap(), b"existing data");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }
}
