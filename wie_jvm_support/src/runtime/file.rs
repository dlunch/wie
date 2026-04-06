use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use wie_backend::System;

use java_runtime::{File, FileSize, FileStat, FileType, IOError, IOResult};

#[derive(Clone)]
pub struct FileImpl {
    path: String,
    write: bool,
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
            write,
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
        let read = filesystem.read(&self.path, cursor, buf.len(), buf).ok_or(IOError::NotFound)?;

        self.cursor.fetch_add(read as u64, Ordering::SeqCst);

        Ok(read)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        if !self.write {
            return Err(IOError::Unsupported);
        }

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
        if !self.write {
            return Err(IOError::Unsupported);
        }

        self.system.filesystem().truncate(&self.path, len as usize);

        Ok(())
    }

    async fn metadata(&self) -> IOResult<FileStat> {
        let size = self.system.filesystem().size(&self.path).ok_or(IOError::NotFound)?;

        Ok(FileStat {
            size: size as _,
            r#type: FileType::File,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use java_runtime::{File, IOError};
    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};

    use super::FileImpl;

    fn new_system() -> System {
        System::new(Box::new(TestPlatform::new()), "test", "test", DefaultTaskRunner)
    }

    #[futures_test::test]
    async fn read_only_files_reject_mutation() {
        let system = new_system();
        system.filesystem().add("readonly.bin", vec![1, 2, 3]);

        let mut file = FileImpl::new(system.clone(), "readonly.bin", false).unwrap();

        assert!(matches!(file.write(&[9]).await, Err(IOError::Unsupported)));
        assert!(matches!(file.set_len(1).await, Err(IOError::Unsupported)));
        assert_eq!(system.filesystem().size("readonly.bin"), Some(3));
    }

    #[futures_test::test]
    async fn writable_files_can_create_and_truncate() {
        let system = new_system();
        let mut file = FileImpl::new(system.clone(), "writeable.bin", true).unwrap();

        assert_eq!(file.write(&[1, 2, 3, 4]).await.unwrap(), 4);
        file.seek(2).await.unwrap();
        assert_eq!(file.write(&[9]).await.unwrap(), 1);
        file.set_len(3).await.unwrap();

        let mut reopened = FileImpl::new(system.clone(), "writeable.bin", false).unwrap();
        let mut buf = [0; 3];
        assert_eq!(reopened.read(&mut buf).await.unwrap(), 3);
        assert_eq!(buf, [1, 2, 9]);
    }
}
