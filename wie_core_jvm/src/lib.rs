#![no_std]
extern crate alloc;

use alloc::{boxed::Box, format, string::String, sync::Arc, vec};
use core::{future::ready, time::Duration};
use wie_skvm::SKVMJavaContextBase;

use bytemuck::cast_vec;

use java_class_proto::MethodBody;
use java_runtime::{classes::java::lang::String as JavaString, File, FileStat, IOError, Runtime};
use jvm::{runtime::JavaLangString, ClassInstance, ClassInstanceRef, JavaError, Jvm, JvmCallback, Result as JvmResult};
use jvm_rust::{ClassDefinitionImpl, JvmDetailImpl};

use wie_backend::{AsyncCallable, System};
use wie_midp::MIDPJavaContextBase;
use wie_wipi_java::WIPIJavaContextBase;

// TODO i think we can merge runtime implementation across platforms..
#[derive(Clone)]
struct JvmCoreRuntime {
    system: System,
    jvm: Arc<Jvm>,
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

    fn spawn(&self, callback: Box<dyn JvmCallback>) {
        struct SpawnProxy {
            jvm: Arc<Jvm>,
            callback: Box<dyn JvmCallback>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, JavaError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                self.callback.call(&self.jvm, vec![].into_boxed_slice()).await?;

                Ok(0) // TODO
            }
        }

        self.system.clone().spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
        });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
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

    async fn open(&self, _path: &str) -> Result<Box<dyn File>, IOError> {
        todo!()
    }

    async fn stat(&self, _path: &str) -> Result<FileStat, IOError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct JvmCore {
    jvm: Arc<Jvm>,
}

impl JvmCore {
    pub async fn new(system: &System) -> JvmResult<Self> {
        let jvm = Arc::new(Jvm::new(JvmDetailImpl).await?);

        let context: Box<dyn Runtime> = Box::new(JvmCoreRuntime {
            system: system.clone(),
            jvm: jvm.clone(),
        });

        java_runtime::initialize(&jvm, move |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, context.clone())) as Box<_>)
        })
        .await?;

        let context: Box<dyn WIPIJavaContextBase> = Box::new(JvmCoreContext {
            system: system.clone(),
            jvm: jvm.clone(),
        });
        wie_wipi_java::register(&jvm, move |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, context.clone())) as Box<_>)
        })
        .await?;

        let context: Box<dyn MIDPJavaContextBase> = Box::new(JvmCoreContext {
            system: system.clone(),
            jvm: jvm.clone(),
        });
        wie_midp::register(&jvm, move |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, context.clone())) as Box<_>)
        })
        .await?;

        // TODO should we add skvm only on skt?
        let context: Box<dyn SKVMJavaContextBase> = Box::new(JvmCoreContext {
            system: system.clone(),
            jvm: jvm.clone(),
        });
        wie_skvm::register(&jvm, move |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, context.clone())) as Box<_>)
        })
        .await?;

        // set initial properties
        let file_encoding_name = JavaLangString::from_rust_string(&jvm, "file.encoding").await?;
        let encoding_str = JavaLangString::from_rust_string(&jvm, "EUC-KR").await?; // TODO hardcoded
        let _: Option<Box<dyn ClassInstance>> = jvm
            .invoke_static(
                "java/lang/System",
                "setProperty",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                (file_encoding_name, encoding_str),
            )
            .await?;

        Ok(Self { jvm })
    }

    pub async fn add_jar(&self, jar: &[u8]) -> JvmResult<Option<String>> {
        let mut storage = self.jvm.instantiate_array("B", jar.len()).await?;
        self.jvm.store_byte_array(&mut storage, 0, cast_vec(jar.to_vec())).await?;

        let class_loader = self.jvm.get_system_class_loader().await?;
        let jar_main_class: ClassInstanceRef<JavaString> = self
            .jvm
            .invoke_virtual(&class_loader, "addJarFile", "([B)Ljava/lang/String;", (storage,))
            .await?;

        if !jar_main_class.is_null() {
            Ok(Some(JavaLangString::to_rust_string(&self.jvm, &jar_main_class).await?))
        } else {
            Ok(None)
        }
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
    jvm: Arc<Jvm>,
}

impl WIPIJavaContextBase for JvmCoreContext {
    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
        self.system.spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
            context: Box::new(self.clone()),
        });

        Ok(())
    }
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
    jvm: Arc<Jvm>,
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
