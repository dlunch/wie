#![no_std]
extern crate alloc;

use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};
use core::{future::ready, time::Duration};

use bytemuck::cast_vec;

use java_class_proto::MethodBody;
use java_runtime::{classes::java::lang::String as JavaString, Runtime};
use jvm::{runtime::JavaLangString, ClassInstanceRef, JavaError, Jvm, JvmCallback, Result as JvmResult};
use jvm_rust::{ClassDefinitionImpl, JvmDetailImpl};

use wie_backend::{AsyncCallable, System};
use wie_wipi_java::WIPIJavaContextBase;

// TODO i think we can merge runtime implementation across platforms..
#[derive(Clone)]
struct JvmCoreRuntime {
    system: System,
    jvm: Rc<Jvm>,
}

#[async_trait::async_trait(?Send)]
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
            jvm: Rc<Jvm>,
            callback: Box<dyn JvmCallback>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, JavaError> for SpawnProxy {
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

    fn encode_str(&self, s: &str) -> Vec<u8> {
        self.system.encode_str(s)
    }

    fn decode_str(&self, bytes: &[u8]) -> String {
        self.system.decode_str(bytes)
    }

    fn println(&mut self, s: &str) {
        tracing::info!("println {}", s);
    }
}

#[derive(Clone)]
pub struct JvmCore {
    jvm: Rc<Jvm>,
}

impl JvmCore {
    pub async fn new(system: &System) -> JvmResult<Self> {
        let jvm = Rc::new(Jvm::new(JvmDetailImpl).await?);

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

    pub fn jvm(&self) -> &Jvm {
        &self.jvm
    }
}

#[derive(Clone)]
struct JvmCoreContext {
    system: System,
    jvm: Rc<Jvm>,
}

impl WIPIJavaContextBase for JvmCoreContext {
    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
        struct SpawnProxy {
            system: System,
            jvm: Rc<Jvm>,
            callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, JavaError> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                let mut context = JvmCoreContext {
                    system: self.system.clone(),
                    jvm: self.jvm.clone(),
                };

                let _ = self.callback.call(&self.jvm, &mut context, Box::new([])).await?;

                Ok(0) // TODO resturn value
            }
        }

        self.system.spawn(SpawnProxy {
            system: self.system.clone(),
            jvm: self.jvm.clone(),
            callback,
        });

        Ok(())
    }
}
