use alloc::{boxed::Box, string::String, sync::Arc};
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
        if !write && !system.filesystem().exists(path) {
            return Err(IOError::NotFound);
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
        let fs = self.system.filesystem();

        let read = fs.read(&self.path, cursor, buf.len(), buf).ok_or(IOError::NotFound)?;

        self.cursor.fetch_add(read as u64, Ordering::SeqCst);

        Ok(read)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        if !self.write {
            return Err(IOError::Unsupported);
        }

        let cursor = self.cursor.load(Ordering::SeqCst) as usize;
        let written = self.system.filesystem().write(&self.path, cursor, buf);

        self.cursor.fetch_add(written as u64, Ordering::SeqCst);

        Ok(written)
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

    use java_runtime::{File, FileType, IOError};
    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};

    use super::FileImpl;

    fn new_system() -> System {
        System::new(Box::new(TestPlatform::new()), "test", "test-aid", DefaultTaskRunner)
    }

    #[futures_test::test]
    async fn virtual_archive_file_is_readable() {
        let system = new_system();
        system.filesystem().add_virtual("res.png", vec![1, 2, 3]);

        let mut file = FileImpl::new(system.clone(), "res.png", false).unwrap();
        let mut buf = [0u8; 3];
        assert_eq!(file.read(&mut buf).await.unwrap(), 3);
        assert_eq!(buf, [1, 2, 3]);

        assert!(matches!(file.write(&[9]).await, Err(IOError::Unsupported)));
        assert!(matches!(file.set_len(1).await, Err(IOError::Unsupported)));
    }

    #[futures_test::test]
    async fn virtual_archive_file_can_be_shadowed() {
        let system = new_system();
        system.filesystem().add_virtual("cfg.dat", vec![0xAA, 0xBB, 0xCC]);

        let mut file = FileImpl::new(system.clone(), "cfg.dat", true).unwrap();
        assert_eq!(file.write(&[1, 2, 3, 4]).await.unwrap(), 4);

        let mut reopened = FileImpl::new(system.clone(), "cfg.dat", false).unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(reopened.read(&mut buf).await.unwrap(), 4);
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn write_handle_sees_virtual_until_first_write() {
        let system = new_system();
        system.filesystem().add_virtual("big.bin", vec![7u8; 10]);

        let mut file = FileImpl::new(system.clone(), "big.bin", true).unwrap();
        let mut buf = [0u8; 10];
        assert_eq!(file.read(&mut buf).await.unwrap(), 10);
        assert_eq!(buf, [7u8; 10]);
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

    #[futures_test::test]
    async fn read_missing_file_returns_not_found() {
        let system = new_system();
        let result = FileImpl::new(system, "nope.dat", false);
        assert!(matches!(result, Err(IOError::NotFound)));
    }

    #[futures_test::test]
    async fn metadata_overlay_size() {
        let system = new_system();
        system.filesystem().add_virtual("f.bin", vec![0u8; 5]);

        {
            let mut writer = FileImpl::new(system.clone(), "f.bin", true).unwrap();
            writer.write(&[1u8; 10]).await.unwrap();
        }

        let file = FileImpl::new(system.clone(), "f.bin", false).unwrap();
        let meta = file.metadata().await.unwrap();
        assert_eq!(meta.size, 10);
        assert!(matches!(meta.r#type, FileType::File));
    }

    #[futures_test::test]
    async fn metadata_falls_back_to_virtual() {
        let system = new_system();
        system.filesystem().add_virtual("only_virtual.bin", vec![0u8; 7]);

        let file = FileImpl::new(system, "only_virtual.bin", false).unwrap();
        let meta = file.metadata().await.unwrap();
        assert_eq!(meta.size, 7);
    }

    #[futures_test::test]
    async fn path_aliases_resolve_to_same_file() {
        let system = new_system();
        system.filesystem().add_virtual("/leading.bin", vec![1, 2, 3, 4]);

        let mut f = FileImpl::new(system.clone(), "./leading.bin", false).unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(f.read(&mut buf).await.unwrap(), 4);
        assert_eq!(buf, [1, 2, 3, 4]);

        let mut f2 = FileImpl::new(system, "/leading.bin", false).unwrap();
        let mut buf2 = [0u8; 4];
        assert_eq!(f2.read(&mut buf2).await.unwrap(), 4);
        assert_eq!(buf2, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn traversal_path_rejected_when_reading() {
        let system = new_system();
        assert!(matches!(FileImpl::new(system.clone(), "../escape.dat", false), Err(IOError::NotFound)));
        assert!(matches!(FileImpl::new(system, "", false), Err(IOError::NotFound)));
    }
}
