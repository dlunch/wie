#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::{
    fs::{self, DirBuilder},
    io::{ErrorKind, Write},
    path::PathBuf,
};

use tempfile::NamedTempFile;

use wie_core_arm_native::NativeCache;
use wie_util::{Result, WieError};

pub(super) struct AotCache {
    pub(super) directory: PathBuf,
}

// SAFETY: Only compiler-produced artifacts are written here, in the host's private cache, outside guest storage.
unsafe impl NativeCache for AotCache {
    fn load(&mut self, key: &[u8; 32]) -> Result<Option<Vec<u8>>> {
        let filename: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
        match fs::read(self.directory.join(filename)) {
            Ok(artifact) => Ok(Some(artifact)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(WieError::FatalError(error.to_string())),
        }
    }

    /// # Safety
    /// `artifact` must be produced by the native compiler/cache pipeline for `key`.
    unsafe fn store(&mut self, key: &[u8; 32], artifact: &[u8]) -> Result<()> {
        let mut directory = DirBuilder::new();
        directory.recursive(true);
        #[cfg(unix)]
        directory.mode(0o700);
        directory
            .create(&self.directory)
            .map_err(|error| WieError::FatalError(error.to_string()))?;

        let filename: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
        let mut file = NamedTempFile::new_in(&self.directory).map_err(|error| WieError::FatalError(error.to_string()))?;
        file.write_all(artifact).map_err(|error| WieError::FatalError(error.to_string()))?;
        file.persist(self.directory.join(filename))
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
    fn missing_cache_is_lazy_and_storage_errors_are_reported() {
        let root = tempdir().unwrap();
        let mut cache = AotCache {
            directory: root.path().join("native-aot"),
        };
        assert!(cache.load(&[0; 32]).unwrap().is_none());
        assert!(!cache.directory.exists());

        fs::create_dir_all(cache.directory.join("00".repeat(32))).unwrap();
        assert!(matches!(cache.load(&[0; 32]), Err(WieError::FatalError(_))));
    }
}
