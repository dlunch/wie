use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::time::Duration;

use spin::Mutex;

use java_runtime::{File, FileSize, FileStat, FileType, IOError, IOResult, RT_RUSTJAR, Runtime, SpawnCallback, get_runtime_class_proto};
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

use crate::{JvmImplementation, JvmSupport, WIE_RUSTJAR, WieJavaClassProto, WieJvmContext};

mod file;

use file::FileImpl;

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

        impl AsyncCallable<Result<(), WieError>> for SpawnProxy {
            async fn call(self) -> Result<(), WieError> {
                let result = self.callback.call().await;
                if let Err(err) = result {
                    return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
                }

                Ok(())
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

    async fn open(&self, path: &str, write: bool) -> Result<Box<dyn File>, IOError> {
        tracing::debug!("open({:?}, {:?})", path, write);

        Ok(Box::new(FileImpl::new(self.system.clone(), path, write)?))
    }

    async fn unlink(&self, _path: &str) -> Result<(), IOError> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self, path: &str) -> IOResult<FileStat> {
        let filesystem = self.system.filesystem();

        if path.is_empty() || path.ends_with("/") {
            return Ok(FileStat {
                size: 0,
                r#type: FileType::Directory,
            });
        }

        let size = filesystem.size(path);
        if let Some(size) = size {
            Ok(FileStat {
                size: size as _,
                r#type: FileType::File,
            })
        } else {
            Err(IOError::NotFound)
        }
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
