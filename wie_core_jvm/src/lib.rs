#![no_std]
extern crate alloc;

use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use core::{future::ready, time::Duration};

use bytemuck::cast_vec;

use java_class_proto::{JavaResult, MethodBody};
use java_runtime::Runtime;
use jvm::{Jvm, JvmCallback, JvmResult};
use jvm_rust::{ClassDefinitionImpl, JvmDetailImpl};

use wie_backend::SystemHandle;
use wie_wipi_java::WIPIJavaContextBase;

pub type JvmCoreResult<T> = anyhow::Result<T>;

#[derive(Clone)]
struct JvmCoreRuntime;

#[async_trait::async_trait(?Send)]
impl Runtime for JvmCoreRuntime {
    async fn sleep(&self, _duration: Duration) {
        todo!()
    }
    async fn r#yield(&self) {
        todo!()
    }

    fn spawn(&self, _callback: Box<dyn JvmCallback>) {
        todo!()
    }

    fn now(&self) -> u64 {
        todo!()
    }

    fn encode_str(&self, _s: &str) -> Vec<u8> {
        todo!()
    }

    fn decode_str(&self, _bytes: &[u8]) -> String {
        todo!()
    }

    fn println(&mut self, _s: &str) {
        todo!()
    }
}

#[derive(Clone)]
pub struct JvmCore {
    jvm: Rc<Jvm>,
}

impl JvmCore {
    pub async fn new(system: &SystemHandle) -> JvmResult<Self> {
        let jvm = Jvm::new(JvmDetailImpl).await?;

        java_runtime::initialize(&jvm, |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, Box::new(JvmCoreRuntime) as Box<_>)) as Box<_>)
        })
        .await?;

        wie_wipi_java::register(&jvm, |name, proto| {
            ready(Box::new(ClassDefinitionImpl::from_class_proto(
                name,
                proto,
                Box::new(JvmCoreContext { system: system.clone() }) as Box<_>,
            )) as Box<_>)
        })
        .await?;

        Ok(Self { jvm: Rc::new(jvm) })
    }

    pub async fn add_jar(&self, jar: &[u8]) -> JvmResult<()> {
        let mut storage = self.jvm.instantiate_array("B", jar.len()).await?;
        self.jvm.store_byte_array(&mut storage, 0, cast_vec(jar.to_vec()))?;

        let class_loader = self.jvm.get_system_class_loader().await?;
        self.jvm.invoke_virtual(&class_loader, "addJarFile", "([B)V", (storage,)).await?;

        Ok(())
    }

    pub fn jvm(&self) -> &Jvm {
        &self.jvm
    }
}

#[derive(Clone)]
struct JvmCoreContext {
    system: SystemHandle,
}

impl WIPIJavaContextBase for JvmCoreContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, _callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>) -> JavaResult<()> {
        todo!()
    }
}
