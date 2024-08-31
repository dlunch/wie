#![no_std]
extern crate alloc;

mod context;

use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use java_runtime::{File, FileStat, IOError, Runtime, RuntimeClassProto, SpawnCallback};
use jvm::{runtime::JavaLangString, ClassDefinition, JavaError, Jvm, Result as JvmResult};
use jvm_rust::{ArrayClassDefinitionImpl, ClassDefinitionImpl};

use wie_backend::{AsyncCallable, System};

pub use context::{WieJavaClassProto, WieJvmContext};

#[derive(Clone)]
struct JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    system: System,
    implementation: T,
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

    fn spawn(&self, callback: Box<dyn SpawnCallback>) {
        struct SpawnProxy {
            callback: Box<dyn SpawnCallback>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, JavaError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                self.callback.call().await;

                Ok(0) // TODO
            }
        }

        self.system.clone().spawn(SpawnProxy { callback });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn current_task_id(&self) -> u64 {
        self.system.current_task_id()
    }

    fn stdin(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> Result<Box<dyn File>, IOError> {
        // can we merge this implementation with ktf's one?
        #[derive(Clone)]
        struct StdoutFile {
            system: System,
        }

        #[async_trait::async_trait]
        impl File for StdoutFile {
            async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, IOError> {
                Err(IOError::Unsupported)
            }

            async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
                self.system.platform().write_stdout(buf);

                Ok(buf.len())
            }
        }

        Ok(Box::new(StdoutFile { system: self.system.clone() }))
    }

    fn stderr(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    async fn open(&self, path: &str) -> Result<Box<dyn File>, IOError> {
        #[derive(Clone)]
        struct FileImpl {
            data: Arc<Vec<u8>>,
            cursor: Arc<AtomicU64>,
        }

        #[async_trait::async_trait]
        impl File for FileImpl {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
                let cursor = self.cursor.load(Ordering::SeqCst) as usize;

                if cursor < self.data.len() {
                    let to_read = core::cmp::min(buf.len(), self.data.len() - cursor);
                    buf[..to_read].copy_from_slice(&self.data[cursor..cursor + to_read]);

                    self.cursor.fetch_add(to_read as u64, Ordering::SeqCst);

                    Ok(to_read)
                } else {
                    Ok(0)
                }
            }

            async fn write(&mut self, _buf: &[u8]) -> Result<usize, IOError> {
                Err(IOError::Unsupported)
            }
        }

        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);

        file.map(|x| {
            Box::new(FileImpl {
                data: Arc::new(x.to_vec()),
                cursor: Arc::new(AtomicU64::new(0)),
            }) as _
        })
        .ok_or(IOError::NotFound)
    }

    async fn stat(&self, path: &str) -> Result<FileStat, IOError> {
        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);

        file.map(|x| FileStat { size: x.len() as _ }).ok_or(IOError::NotFound)
    }

    async fn define_class_rust(&self, jvm: &Jvm, proto: RuntimeClassProto) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_class_rust(jvm, proto, Box::new(self.clone()) as Box<_>).await
    }

    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_class_java(jvm, data).await
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_array_class(_jvm, element_type_name).await
    }
}

pub struct JvmSupport {}

impl JvmSupport {
    pub async fn new_jvm<T>(system: &System, jar_name: Option<&str>, protos: Box<[Box<[WieJavaClassProto]>]>, implementation: T) -> JvmResult<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let runtime = JvmRuntime {
            system: system.clone(),
            implementation: implementation.clone(),
        };

        let properties = [("file.encoding", "EUC-KR"), ("java.class.path", jar_name.unwrap_or(""))]
            .into_iter()
            .collect();
        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await?;
        let context = Box::new(WieJvmContext::new(system));

        for proto in protos.into_vec().into_iter().flat_map(|x| x.into_vec()) {
            let class = implementation.define_class_wie(&jvm, proto, context.clone()).await?;

            jvm.register_class(class, None).await?;
            // TODO add class loader
        }

        Ok(jvm)
    }

    pub async fn format_err(jvm: &Jvm, err: JavaError) -> String {
        if let JavaError::JavaException(x) = err {
            let to_string = jvm.invoke_virtual(&x, "toString", "()Ljava/lang/String;", ()).await.unwrap();

            JavaLangString::to_rust_string(jvm, &to_string).await.unwrap()
        } else {
            format!("{:?}", err)
        }
    }
}

#[async_trait::async_trait]
pub trait JvmImplementation: Clone {
    async fn define_class_wie(&self, jvm: &Jvm, proto: WieJavaClassProto, context: Box<WieJvmContext>) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_class_rust(&self, jvm: &Jvm, proto: RuntimeClassProto, runtime: Box<dyn Runtime>) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>>;
}

#[derive(Clone)]
pub struct RustJavaJvmImplementation;

#[async_trait::async_trait]
impl JvmImplementation for RustJavaJvmImplementation {
    async fn define_class_wie(&self, _jvm: &Jvm, proto: WieJavaClassProto, context: Box<WieJvmContext>) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, context)))
    }

    async fn define_class_rust(&self, _jvm: &Jvm, proto: RuntimeClassProto, runtime: Box<dyn Runtime>) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, runtime)))
    }

    async fn define_class_java(&self, _jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        ClassDefinitionImpl::from_classfile(data).map(|x| Box::new(x) as Box<_>)
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}
