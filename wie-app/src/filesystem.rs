use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};

use wie_backend::Filesystem;

/// Persistent guest files under the Tauri app data directory's `<aid>/fs`.
pub(crate) struct DiskFilesystem {
    base_path: PathBuf,
}

impl DiskFilesystem {
    pub(crate) fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn path_for(&self, aid: &str, path: &str) -> Option<PathBuf> {
        let mut has_name = false;
        for component in Path::new(path).components() {
            match component {
                Component::Normal(_) => has_name = true,
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    tracing::error!(aid, path, "path traversal attempt rejected");
                    return None;
                }
            }
        }

        if !has_name {
            tracing::error!(aid, path, "rejected: empty normalized path");
            return None;
        }

        Some(self.base_path.join(aid).join("fs").join(path))
    }
}

#[async_trait::async_trait]
impl Filesystem for DiskFilesystem {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        let Some(disk_path) = self.path_for(aid, path) else {
            return false;
        };

        match disk_path.metadata() {
            Ok(metadata) => metadata.is_file(),
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(aid, path, %error, "Failed to check file existence");
                }
                false
            }
        }
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        let disk_path = self.path_for(aid, path)?;
        let metadata = disk_path
            .metadata()
            .inspect_err(|error| {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(aid, path, %error, "Failed to read file size");
                }
            })
            .ok()?;
        if !metadata.is_file() {
            return None;
        }
        Some(metadata.len() as usize)
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let disk_path = self.path_for(aid, path)?;
        let mut file = match OpenOptions::new().read(true).open(&disk_path) {
            Ok(file) => file,
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    tracing::warn!(aid, path, %error, "read: open failed");
                }
                return None;
            }
        };

        let size = file.metadata().map(|metadata| metadata.len() as usize).unwrap_or_else(|error| {
            tracing::warn!(aid, path, %error, "read: metadata failed");
            0
        });
        if offset >= size {
            return Some(0);
        }
        if let Err(error) = file.seek(SeekFrom::Start(offset as u64)) {
            tracing::warn!(aid, path, %error, "read: seek failed");
            return Some(0);
        }

        let to_read = count.min(size - offset);
        match file.read_exact(&mut buf[..to_read]) {
            Ok(()) => Some(to_read),
            Err(error) => {
                tracing::warn!(aid, path, %error, "read: IO error");
                Some(0)
            }
        }
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let Some(disk_path) = self.path_for(aid, path) else {
            return 0;
        };
        if let Some(parent) = disk_path.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            tracing::warn!(aid, path, %error, "write: create parent dir failed");
            return 0;
        }

        let mut file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&disk_path) {
            Ok(file) => file,
            Err(error) => {
                tracing::warn!(aid, path, %error, "write: open failed");
                return 0;
            }
        };
        let current_size = match file.metadata() {
            Ok(metadata) => metadata.len() as usize,
            Err(error) => {
                tracing::warn!(aid, path, %error, "write: metadata failed");
                return 0;
            }
        };
        // Preserve zero-filled gaps even when the write contains no bytes.
        if offset > current_size
            && let Err(error) = file.set_len(offset as u64)
        {
            tracing::warn!(aid, path, %error, "write: set_len extend failed");
            return 0;
        }
        if let Err(error) = file.seek(SeekFrom::Start(offset as u64)) {
            tracing::warn!(aid, path, %error, "write: seek failed");
            return 0;
        }

        match file.write_all(data) {
            Ok(()) => data.len(),
            Err(error) => {
                tracing::warn!(aid, path, %error, "write: write_all failed");
                0
            }
        }
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        let Some(disk_path) = self.path_for(aid, path) else {
            return;
        };
        if let Some(parent) = disk_path.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            tracing::warn!(aid, path, %error, "truncate: create parent dir failed");
            return;
        }
        let file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&disk_path) {
            Ok(file) => file,
            Err(error) => {
                tracing::warn!(aid, path, %error, "truncate: open failed");
                return;
            }
        };
        if let Err(error) = file.set_len(len as u64) {
            tracing::warn!(aid, path, %error, "truncate: set_len failed");
        }
    }
}
