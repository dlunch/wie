use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use wie_backend::System;

use java_runtime::{File, FileSize, FileStat, FileType, IOError, IOResult};

#[derive(Clone)]
pub struct FileImpl {
    path: String,
    cursor: Arc<AtomicU64>,
    system: System,
}

impl FileImpl {
    pub fn new(system: System, path: &str, write: bool) -> Result<Self, IOError> {
        {
            let mut filesystem = system.filesystem();

            if write {
                if !filesystem.exists(path) {
                    filesystem.add(path, Vec::new());
                }
            } else if !filesystem.exists(path) {
                return Err(IOError::NotFound);
            }
        }

        Ok(Self {
            path: path.into(),
            cursor: Arc::new(AtomicU64::new(0)),
            system,
        })
    }
}

#[async_trait::async_trait]
impl File for FileImpl {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        let cursor = self.cursor.load(Ordering::SeqCst) as usize;

        let filesystem = self.system.filesystem();
        let read = filesystem.read(&self.path, cursor, buf.len(), buf).unwrap();

        self.cursor.fetch_add(read as u64, Ordering::SeqCst);

        Ok(read)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        let cursor = self.cursor.load(Ordering::SeqCst) as usize;

        let mut filesystem = self.system.filesystem();
        let write = filesystem.write(&self.path, cursor, buf);

        self.cursor.fetch_add(write as u64, Ordering::SeqCst);

        Ok(write)
    }

    async fn seek(&mut self, pos: FileSize) -> IOResult<()> {
        self.cursor.store(pos, Ordering::SeqCst);

        Ok(())
    }

    async fn tell(&self) -> IOResult<FileSize> {
        Ok(self.cursor.load(Ordering::SeqCst))
    }

    async fn set_len(&mut self, len: FileSize) -> IOResult<()> {
        self.system.filesystem().truncate(&self.path, len as usize);

        Ok(())
    }

    async fn metadata(&self) -> IOResult<FileStat> {
        let size = self.system.filesystem().size(&self.path).unwrap();

        Ok(FileStat {
            size: size as _,
            r#type: FileType::File,
        })
    }
}
