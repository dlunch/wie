use alloc::{boxed::Box, string::String, sync::Arc};
use core::sync::atomic::{AtomicU64, Ordering};

use wie_backend::System;

use java_runtime::{File, FileOpenOptions, FileSize, FileStat, FileType, IOError, IOResult};

#[derive(Clone)]
pub struct FileImpl {
    path: String,
    options: FileOpenOptions,
    cursor: Arc<AtomicU64>,
    system: System,
}

impl FileImpl {
    pub async fn new(system: System, path: &str, options: FileOpenOptions) -> Result<Self, IOError> {
        let filesystem = system.filesystem();
        let exists = filesystem.exists(path).await;
        if !exists && !options.create {
            return Err(IOError::NotFound);
        }

        if (options.truncate || !exists) && !filesystem.truncate(path, 0).await {
            return Err(IOError::Io);
        }

        let cursor = if options.append {
            filesystem.size(path).await.ok_or(IOError::NotFound)? as u64
        } else {
            0
        };

        Ok(Self {
            path: path.into(),
            options,
            cursor: Arc::new(AtomicU64::new(cursor)),
            system,
        })
    }
}

#[async_trait::async_trait]
impl File for FileImpl {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        if !self.options.read {
            return Err(IOError::Unsupported);
        }

        let cursor = self.cursor.load(Ordering::SeqCst) as usize;
        let fs = self.system.filesystem();

        let read = fs.read(&self.path, cursor, buf.len(), buf).await.ok_or(IOError::NotFound)?;

        self.cursor.fetch_add(read as u64, Ordering::SeqCst);

        Ok(read)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        if !self.options.write && !self.options.append {
            return Err(IOError::Unsupported);
        }

        let filesystem = self.system.filesystem();
        let cursor = if self.options.append {
            filesystem.size(&self.path).await.ok_or(IOError::NotFound)?
        } else {
            self.cursor.load(Ordering::SeqCst) as usize
        };
        let written = filesystem.write(&self.path, cursor, buf).await;
        if written != buf.len() {
            return Err(IOError::Io);
        }

        self.cursor.store((cursor + written) as u64, Ordering::SeqCst);

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
        if !self.options.write && !self.options.append {
            return Err(IOError::Unsupported);
        }

        if !self.system.filesystem().truncate(&self.path, len as usize).await {
            return Err(IOError::Io);
        }

        Ok(())
    }

    async fn metadata(&self) -> IOResult<FileStat> {
        let size = self.system.filesystem().size(&self.path).await.ok_or(IOError::NotFound)?;

        Ok(FileStat {
            size: size as _,
            r#type: FileType::File,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use java_runtime::{File, FileOpenOptions, FileType, IOError};
    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};

    use super::FileImpl;

    const READ_ONLY: FileOpenOptions = FileOpenOptions {
        read: true,
        write: false,
        append: false,
        truncate: false,
        create: false,
    };
    const WRITE_ONLY: FileOpenOptions = FileOpenOptions {
        read: false,
        write: true,
        append: false,
        truncate: false,
        create: false,
    };
    const READ_WRITE: FileOpenOptions = FileOpenOptions {
        read: true,
        write: true,
        append: false,
        truncate: false,
        create: false,
    };
    const READ_WRITE_CREATE: FileOpenOptions = FileOpenOptions { create: true, ..READ_WRITE };
    const WRITE_CREATE: FileOpenOptions = FileOpenOptions { create: true, ..WRITE_ONLY };
    const WRITE_TRUNCATE_CREATE: FileOpenOptions = FileOpenOptions {
        truncate: true,
        create: true,
        ..WRITE_ONLY
    };
    const WRITE_APPEND_CREATE: FileOpenOptions = FileOpenOptions {
        append: true,
        create: true,
        ..WRITE_ONLY
    };

    fn new_system() -> System {
        System::new(Box::new(TestPlatform::new()), "test", "test-aid", DefaultTaskRunner)
    }

    #[futures_test::test]
    async fn virtual_archive_file_is_readable() {
        let system = new_system();
        system.filesystem().add_virtual("res.png", vec![1, 2, 3]);

        let mut file = FileImpl::new(system.clone(), "res.png", READ_ONLY).await.unwrap();
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

        let mut file = FileImpl::new(system.clone(), "cfg.dat", WRITE_ONLY).await.unwrap();
        assert_eq!(file.write(&[1, 2, 3, 4]).await.unwrap(), 4);

        let mut reopened = FileImpl::new(system.clone(), "cfg.dat", READ_ONLY).await.unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(reopened.read(&mut buf).await.unwrap(), 4);
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn write_handle_sees_virtual_until_first_write() {
        let system = new_system();
        system.filesystem().add_virtual("big.bin", vec![7u8; 10]);

        let mut file = FileImpl::new(system.clone(), "big.bin", READ_WRITE).await.unwrap();
        let mut buf = [0u8; 10];
        assert_eq!(file.read(&mut buf).await.unwrap(), 10);
        assert_eq!(buf, [7u8; 10]);
    }

    #[futures_test::test]
    async fn writable_files_can_create_and_truncate() {
        let system = new_system();
        let mut file = FileImpl::new(system.clone(), "writeable.bin", READ_WRITE_CREATE).await.unwrap();

        assert_eq!(file.write(&[1, 2, 3, 4]).await.unwrap(), 4);
        file.seek(2).await.unwrap();
        assert_eq!(file.write(&[9]).await.unwrap(), 1);
        file.set_len(3).await.unwrap();

        let mut reopened = FileImpl::new(system.clone(), "writeable.bin", READ_ONLY).await.unwrap();
        let mut buf = [0; 3];
        assert_eq!(reopened.read(&mut buf).await.unwrap(), 3);
        assert_eq!(buf, [1, 2, 9]);
    }

    #[futures_test::test]
    async fn open_options_truncate_and_append_without_seeking() {
        let system = new_system();
        system.filesystem().add_virtual("stream.bin", b"old".to_vec());

        let mut writer = FileImpl::new(system.clone(), "stream.bin", WRITE_TRUNCATE_CREATE).await.unwrap();
        assert_eq!(writer.metadata().await.unwrap().size, 0);
        assert!(matches!(writer.read(&mut [0]).await, Err(IOError::Unsupported)));
        writer.write(b"new").await.unwrap();

        let mut first = FileImpl::new(system.clone(), "stream.bin", WRITE_APPEND_CREATE).await.unwrap();
        let mut second = FileImpl::new(system.clone(), "stream.bin", WRITE_APPEND_CREATE).await.unwrap();
        first.seek(0).await.unwrap();
        second.seek(0).await.unwrap();
        first.write(b"-a").await.unwrap();
        second.write(b"-b").await.unwrap();
        first.write(b"-c").await.unwrap();

        let mut reader = FileImpl::new(system, "stream.bin", READ_ONLY).await.unwrap();
        let mut buf = [0; 9];
        assert_eq!(reader.read(&mut buf).await.unwrap(), buf.len());
        assert_eq!(&buf, b"new-a-b-c");
    }

    #[futures_test::test]
    async fn read_missing_file_returns_not_found() {
        let system = new_system();
        let result = FileImpl::new(system, "nope.dat", READ_ONLY).await;
        assert!(matches!(result, Err(IOError::NotFound)));
    }

    #[futures_test::test]
    async fn metadata_overlay_size() {
        let system = new_system();
        system.filesystem().add_virtual("f.bin", vec![0u8; 5]);

        {
            let mut writer = FileImpl::new(system.clone(), "f.bin", WRITE_ONLY).await.unwrap();
            writer.write(&[1u8; 10]).await.unwrap();
        }

        let file = FileImpl::new(system.clone(), "f.bin", READ_ONLY).await.unwrap();
        let meta = file.metadata().await.unwrap();
        assert_eq!(meta.size, 10);
        assert!(matches!(meta.r#type, FileType::File));
    }

    #[futures_test::test]
    async fn metadata_falls_back_to_virtual() {
        let system = new_system();
        system.filesystem().add_virtual("only_virtual.bin", vec![0u8; 7]);

        let file = FileImpl::new(system, "only_virtual.bin", READ_ONLY).await.unwrap();
        let meta = file.metadata().await.unwrap();
        assert_eq!(meta.size, 7);
    }

    #[futures_test::test]
    async fn path_aliases_resolve_to_same_file() {
        let system = new_system();
        system.filesystem().add_virtual("/leading.bin", vec![1, 2, 3, 4]);

        let mut f = FileImpl::new(system.clone(), "./leading.bin", READ_ONLY).await.unwrap();
        let mut buf = [0u8; 4];
        assert_eq!(f.read(&mut buf).await.unwrap(), 4);
        assert_eq!(buf, [1, 2, 3, 4]);

        let mut f2 = FileImpl::new(system, "/leading.bin", READ_ONLY).await.unwrap();
        let mut buf2 = [0u8; 4];
        assert_eq!(f2.read(&mut buf2).await.unwrap(), 4);
        assert_eq!(buf2, [1, 2, 3, 4]);
    }

    #[futures_test::test]
    async fn traversal_path_rejected_when_reading() {
        let system = new_system();
        assert!(matches!(
            FileImpl::new(system.clone(), "../escape.dat", READ_ONLY).await,
            Err(IOError::NotFound)
        ));
        assert!(matches!(FileImpl::new(system, "", READ_ONLY).await, Err(IOError::NotFound)));
    }

    #[futures_test::test]
    async fn invalid_write_path_fails_during_creation() {
        let system = new_system();
        let result = FileImpl::new(system, "../escape.dat", WRITE_CREATE).await;
        assert!(matches!(result, Err(IOError::Io)));
    }
}
