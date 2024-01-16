use alloc::boxed::Box;

use java_class_proto::MethodBody;
use jvm::JvmResult;

use wie_backend::{AsyncCallable, SystemHandle};
use wie_core_arm::ArmCore;
use wie_wipi_java::WIPIJavaContextBase;

use crate::runtime::java::jvm::KtfJvm;

#[derive(Clone)]
pub struct KtfWIPIJavaContext {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfWIPIJavaContext {
    pub fn new(core: &ArmCore, system: &SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl WIPIJavaContextBase for KtfWIPIJavaContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: SystemHandle,
            callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                let mut context = KtfWIPIJavaContext::new(&self.core, &self.system);
                let mut jvm = KtfJvm::new(&self.core, &self.system);

                let _ = self.callback.call(&jvm.jvm(), &mut context, Box::new([])).await?;

                Ok(0) // TODO resturn value
            }
        }

        let system = self.system.clone();

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            system,
            callback,
        });

        Ok(())
    }
}
