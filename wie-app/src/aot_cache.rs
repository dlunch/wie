#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{fs::DirBuilder, path::PathBuf};

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use wie_core_arm_native::NativeCache;
use wie_util::{Result, WieError};

pub(crate) struct AotCache {
    pub(crate) directory: PathBuf,
}

impl NativeCache for AotCache {
    fn load(&mut self, key: &[u8; 32]) -> Result<Option<Vec<u8>>> {
        let path = self.directory.join("native-aot.sqlite");
        if !path.try_exists().map_err(|error| WieError::FatalError(error.to_string()))? {
            return Ok(None);
        }
        let connection =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|error| WieError::FatalError(error.to_string()))?;
        connection
            .query_row("SELECT artifact FROM artifacts WHERE key = ?1", [key], |row| row.get(0))
            .optional()
            .map_err(|error| WieError::FatalError(error.to_string()))
    }

    fn store(&mut self, key: &[u8; 32], artifact: &[u8]) -> Result<()> {
        let mut directory = DirBuilder::new();
        directory.recursive(true);
        #[cfg(unix)]
        directory.mode(0o700);
        directory
            .create(&self.directory)
            .map_err(|error| WieError::FatalError(error.to_string()))?;

        let connection = Connection::open(self.directory.join("native-aot.sqlite")).map_err(|error| WieError::FatalError(error.to_string()))?;
        connection
            .execute_batch("CREATE TABLE IF NOT EXISTS artifacts (key BLOB PRIMARY KEY, artifact BLOB NOT NULL)")
            .map_err(|error| WieError::FatalError(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO artifacts (key, artifact) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET artifact = excluded.artifact",
                params![key, artifact],
            )
            .map_err(|error| WieError::FatalError(error.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use wie_core_arm_native::NativeCache;
    use wie_util::WieError;

    use super::AotCache;

    #[test]
    fn cache_roundtrip_replaces_records_and_reports_storage_errors() {
        let root = tempdir().unwrap();
        let mut cache = AotCache {
            directory: root.path().join("cache"),
        };
        assert!(cache.load(&[0; 32]).unwrap().is_none());
        assert!(!cache.directory.exists());

        for bytes in [b"first".as_slice(), b"replacement".as_slice()] {
            cache.store(&[0; 32], bytes).unwrap();
            assert_eq!(cache.load(&[0; 32]).unwrap().unwrap(), bytes);
        }
        fs::write(cache.directory.join("native-aot.sqlite"), b"not a database").unwrap();
        assert!(matches!(cache.load(&[0; 32]), Err(WieError::FatalError(_))));
    }
}
