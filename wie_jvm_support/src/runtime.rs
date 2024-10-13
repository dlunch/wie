use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use spin::Mutex;

use java_runtime::{get_runtime_class_proto, File, FileSize, FileStat, FileType, IOError, IOResult, Runtime, SpawnCallback, RT_RUSTJAR};
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

use crate::{JvmImplementation, JvmSupport, WieJavaClassProto, WieJvmContext, WIE_RUSTJAR};

#[derive(Clone)]
pub struct JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    system: System,
    implementation: T,
    protos: Arc<Mutex<Vec<WieJavaClassProto>>>,
}

impl<T> JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    pub fn new(system: System, implementation: T, protos: Box<[Box<[WieJavaClassProto]>]>) -> Self {
        Self {
            system,
            implementation,
            protos: Arc::new(Mutex::new(protos.into_vec().into_iter().flat_map(|x| x.into_vec()).collect())),
        }
    }
}

#[async_trait::async_trait]
impl<T> Runtime for JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    async fn sleep(&self, duration: Duration) {
        let now = self.system.platform().now();
        let until = now + duration.as_millis() as u64;

        self.system.clone().sleep(until).await; // TODO remove clone
    }

    async fn r#yield(&self) {
        self.system.yield_now().await;
    }

    fn spawn(&self, jvm: &Jvm, callback: Box<dyn SpawnCallback>) {
        struct SpawnProxy {
            jvm: Jvm,
            callback: Box<dyn SpawnCallback>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, WieError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, WieError> {
                let result = self.callback.call().await;
                if let Err(err) = result {
                    return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
                }

                Ok(0)
            }
        }

        self.system.clone().spawn(SpawnProxy { jvm: jvm.clone(), callback });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn current_task_id(&self) -> u64 {
        self.system.current_task_id()
    }

    fn stdin(&self) -> IOResult<Box<dyn File>> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> IOResult<Box<dyn File>> {
        #[derive(Clone)]
        struct StdoutFile {
            system: System,
        }

        #[async_trait::async_trait]
        impl File for StdoutFile {
            async fn read(&mut self, _buf: &mut [u8]) -> IOResult<usize> {
                Err(IOError::Unsupported)
            }

            async fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
                self.system.platform().write_stdout(buf);

                Ok(buf.len())
            }

            async fn seek(&mut self, _pos: FileSize) -> IOResult<()> {
                Err(IOError::Unsupported)
            }

            async fn tell(&self) -> IOResult<FileSize> {
                Err(IOError::Unsupported)
            }

            async fn set_len(&mut self, _len: FileSize) -> IOResult<()> {
                Err(IOError::Unsupported)
            }

            async fn metadata(&self) -> IOResult<FileStat> {
                Err(IOError::Unsupported)
            }
        }

        Ok(Box::new(StdoutFile { system: self.system.clone() }))
    }

    fn stderr(&self) -> Result<Box<dyn File>, IOError> {
        #[derive(Clone)]
        struct StderrFile {
            system: System,
        }

        #[async_trait::async_trait]
        impl File for StderrFile {
            async fn read(&mut self, _buf: &mut [u8]) -> IOResult<usize> {
                Err(IOError::Unsupported)
            }

            async fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
                self.system.platform().write_stderr(buf);

                Ok(buf.len())
            }

            async fn seek(&mut self, _pos: FileSize) -> IOResult<()> {
                Err(IOError::Unsupported)
            }

            async fn tell(&self) -> IOResult<FileSize> {
                Err(IOError::Unsupported)
            }

            async fn set_len(&mut self, _len: FileSize) -> IOResult<()> {
                Err(IOError::Unsupported)
            }

            async fn metadata(&self) -> IOResult<FileStat> {
                Err(IOError::Unsupported)
            }
        }

        Ok(Box::new(StderrFile { system: self.system.clone() }))
    }

    async fn open(&self, path: &str, write: bool, create: bool) -> Result<Box<dyn File>, IOError> {
        tracing::debug!("open({:?}, {:?}, {:?})", path, write, create);

        #[derive(Clone)]
        struct FileImpl {
            data: Arc<Mutex<Vec<u8>>>,
            cursor: Arc<AtomicU64>,
        }

        #[async_trait::async_trait]
        impl File for FileImpl {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
                let cursor = self.cursor.load(Ordering::SeqCst) as usize;

                let data = self.data.lock();

                if cursor < data.len() {
                    let to_read = core::cmp::min(buf.len(), data.len() - cursor);
                    buf[..to_read].copy_from_slice(&data[cursor..cursor + to_read]);

                    self.cursor.fetch_add(to_read as u64, Ordering::SeqCst);

                    Ok(to_read)
                } else {
                    Ok(0)
                }
            }

            async fn write(&mut self, _buf: &[u8]) -> Result<usize, IOError> {
                Err(IOError::Unsupported)
            }

            async fn seek(&mut self, pos: FileSize) -> IOResult<()> {
                self.cursor.store(pos, Ordering::SeqCst);

                Ok(())
            }

            async fn tell(&self) -> IOResult<FileSize> {
                Err(IOError::Unsupported)
            }

            async fn set_len(&mut self, len: FileSize) -> IOResult<()> {
                self.data.lock().resize(len as usize, 0);

                Ok(())
            }

            async fn metadata(&self) -> IOResult<FileStat> {
                Ok(FileStat {
                    size: self.data.lock().len() as _,
                    r#type: FileType::File,
                })
            }
        }

        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);
        if let Some(x) = file {
            Ok(Box::new(FileImpl {
                data: Arc::new(Mutex::new(x.to_vec())),
                cursor: Arc::new(AtomicU64::new(0)),
            }))
        } else if create {
            Ok(Box::new(FileImpl {
                data: Arc::new(Mutex::new(vec![])),
                cursor: Arc::new(AtomicU64::new(0)),
            }))
        } else {
            Err(IOError::NotFound)
        }
    }

    async fn unlink(&self, _path: &str) -> Result<(), IOError> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self, path: &str) -> IOResult<FileStat> {
        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);

        file.map(|x| FileStat {
            size: x.len() as _,
            r#type: FileType::File,
        })
        .ok_or(IOError::NotFound)
    }

    async fn find_rustjar_class(&self, jvm: &Jvm, classpath: &str, class: &str) -> JvmResult<Option<Box<dyn ClassDefinition>>> {
        if classpath == RT_RUSTJAR {
            let proto = get_runtime_class_proto(class);
            if let Some(proto) = proto {
                return Ok(Some(
                    self.implementation
                        .define_class_rust(jvm, proto, Box::new(self.clone()) as Box<_>)
                        .await?,
                ));
            }
        } else if classpath == WIE_RUSTJAR {
            let proto_index = self.protos.lock().iter().position(|x| x.name == class);
            if let Some(proto_index) = proto_index {
                let proto = self.protos.lock().remove(proto_index);
                let context = Box::new(WieJvmContext::new(&self.system));

                return Ok(Some(self.implementation.define_class_rust(jvm, proto, context as Box<_>).await?));
            }
        }

        Ok(None)
    }

    async fn define_class(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_class_java(jvm, data).await
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_array_class(_jvm, element_type_name).await
    }
}
