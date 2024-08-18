#![no_std]
extern crate alloc;

use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use core::{
    future::ready,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use wie_skvm::SKVMJavaContextBase;
use wie_wipi_java::WIPIJavaContext;

use java_class_proto::MethodBody;
use java_runtime::{File, FileStat, IOError, Runtime, RuntimeClassProto, SpawnCallback};
use jvm::{runtime::JavaLangString, ClassDefinition, JavaError, Jvm, Result as JvmResult};
use jvm_rust::{ArrayClassDefinitionImpl, ClassDefinitionImpl};

use wie_backend::{AsyncCallable, System};
use wie_midp::MIDPJavaContextBase;

// TODO i think we can merge runtime implementation across platforms..
#[derive(Clone)]
struct JvmCoreRuntime {
    system: System,
}

#[async_trait::async_trait]
impl Runtime for JvmCoreRuntime {
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

    async fn define_class_rust(&self, _jvm: &Jvm, proto: RuntimeClassProto) -> jvm::Result<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, Box::new(self.clone()) as Box<_>)))
    }

    async fn define_class_java(&self, _jvm: &Jvm, data: &[u8]) -> jvm::Result<Box<dyn ClassDefinition>> {
        ClassDefinitionImpl::from_classfile(data).map(|x| Box::new(x) as Box<_>)
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> jvm::Result<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}

#[derive(Clone)]
pub struct JvmCore {
    jvm: Jvm,
}

impl JvmCore {
    pub async fn new(system: &System, jar_name: &str) -> JvmResult<Self> {
        let runtime = JvmCoreRuntime { system: system.clone() };

        let properties = [("file.encoding", "EUC-KR"), ("java.class.path", jar_name)].into_iter().collect();
        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await?;
        let context = WIPIJavaContext::new(system);

        wie_wipi_java::register(&jvm, move |proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(proto, Box::new(context.clone()))) as Box<_>)
        })
        .await?;

        let context: Box<dyn MIDPJavaContextBase> = Box::new(JvmCoreContext {
            system: system.clone(),
            jvm: jvm.clone(),
        });
        wie_midp::register(&jvm, move |proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(proto, context.clone())) as Box<_>)
        })
        .await?;

        // TODO should we add skvm only on skt?
        let context: Box<dyn SKVMJavaContextBase> = Box::new(JvmCoreContext {
            system: system.clone(),
            jvm: jvm.clone(),
        });
        wie_skvm::register(&jvm, move |proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(proto, context.clone())) as Box<_>)
        })
        .await?;

        Ok(Self { jvm })
    }

    pub async fn format_err(jvm: &Jvm, err: JavaError) -> String {
        if let JavaError::JavaException(x) = err {
            let to_string = jvm.invoke_virtual(&x, "toString", "()Ljava/lang/String;", ()).await.unwrap();

            JavaLangString::to_rust_string(jvm, &to_string).await.unwrap()
        } else {
            format!("{:?}", err)
        }
    }

    pub fn jvm(&self) -> &Jvm {
        &self.jvm
    }
}

#[derive(Clone)]
struct JvmCoreContext {
    system: System,
    jvm: Jvm,
}

impl MIDPJavaContextBase for JvmCoreContext {
    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, dyn MIDPJavaContextBase>>) -> JvmResult<()> {
        self.system.spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
            context: Box::new(self.clone()),
        });

        Ok(())
    }
}

impl SKVMJavaContextBase for JvmCoreContext {
    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, dyn SKVMJavaContextBase>>) -> JvmResult<()> {
        self.system.spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
            context: Box::new(self.clone()),
        });

        Ok(())
    }
}

struct SpawnProxy<T>
where
    T: ?Sized,
{
    jvm: Jvm,
    callback: Box<dyn MethodBody<JavaError, T>>,
    context: Box<T>,
}

#[async_trait::async_trait]
impl<T> AsyncCallable<Result<u32, JavaError>> for SpawnProxy<T>
where
    T: ?Sized + Send,
{
    async fn call(mut self) -> Result<u32, JavaError> {
        let result = self.callback.call(&self.jvm, &mut self.context, Box::new([])).await;
        if let Err(x) = result {
            let err = JvmCore::format_err(&self.jvm, x).await;
            tracing::error!("Error: {}", err);
        }

        Ok(0) // TODO return value
    }
}
