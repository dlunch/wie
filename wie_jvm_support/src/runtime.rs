use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use core::time::Duration;

use spin::Mutex;

use java_runtime::{
    File, FileDescriptorId, FileSize, FileStat, FileType, IOError, IOResult, RT_RUSTJAR, Runtime, SpawnCallback, get_runtime_class_proto,
};
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

use crate::{JvmImplementation, JvmSupport, WIE_RUSTJAR, WieJavaClassProto, WieJvmContext};

mod file;

use file::FileImpl;

const STDOUT_FD: u32 = 1;
const STDERR_FD: u32 = 2;

struct FileTableInner {
    files: BTreeMap<u32, Box<dyn File>>,
    next_id: u32,
}

impl FileTableInner {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            next_id: 3, // 0=stdin, 1=stdout, 2=stderr
        }
    }

    fn add(&mut self, file: Box<dyn File>) -> FileDescriptorId {
        let id = self.next_id;
        self.next_id += 1;
        self.files.insert(id, file);
        FileDescriptorId::new(id)
    }
}

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

#[derive(Clone)]
pub struct JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    system: System,
    implementation: T,
    protos: Arc<Mutex<Vec<WieJavaClassProto>>>,
    file_table: Arc<Mutex<FileTableInner>>,
}

impl<T> JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    pub fn new(system: System, implementation: T, protos: Box<[Box<[WieJavaClassProto]>]>) -> Self {
        let mut file_table = FileTableInner::new();
        file_table.files.insert(STDOUT_FD, Box::new(StdoutFile { system: system.clone() }));
        file_table.files.insert(STDERR_FD, Box::new(StderrFile { system: system.clone() }));

        Self {
            system,
            implementation,
            protos: Arc::new(Mutex::new(protos.into_vec().into_iter().flat_map(|x| x.into_vec()).collect())),
            file_table: Arc::new(Mutex::new(file_table)),
        }
    }
}

#[async_trait::async_trait]
impl<T> Runtime for JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    async fn sleep(&self, duration: Duration) {
        self.system.sleep(duration.as_millis() as _).await;
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

        self.system.spawn(SpawnProxy { jvm: jvm.clone(), callback });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn current_task_id(&self) -> u64 {
        self.system.current_task_id()
    }

    fn stdin(&self) -> IOResult<FileDescriptorId> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> IOResult<FileDescriptorId> {
        Ok(FileDescriptorId::new(STDOUT_FD))
    }

    fn stderr(&self) -> IOResult<FileDescriptorId> {
        Ok(FileDescriptorId::new(STDERR_FD))
    }

    async fn open(&self, path: &str, write: bool) -> IOResult<FileDescriptorId> {
        tracing::debug!("open({path:?}, {write:?})");

        let file = FileImpl::new(self.system.clone(), path, write).await?;
        Ok(self.file_table.lock().add(Box::new(file)))
    }

    fn get_file(&self, fd: FileDescriptorId) -> IOResult<Box<dyn File>> {
        self.file_table.lock().files.get(&fd.id()).cloned().ok_or(IOError::NotFound)
    }

    fn close_file(&self, fd: FileDescriptorId) {
        self.file_table.lock().files.remove(&fd.id());
    }

    async fn unlink(&self, _path: &str) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self, path: &str) -> IOResult<FileStat> {
        if path.is_empty() || path.ends_with("/") {
            return Ok(FileStat {
                size: 0,
                r#type: FileType::Directory,
            });
        }

        let size = self.system.filesystem().size(path).await.ok_or(IOError::NotFound)?;

        Ok(FileStat {
            size: size as _,
            r#type: FileType::File,
        })
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
