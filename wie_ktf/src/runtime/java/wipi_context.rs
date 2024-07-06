use alloc::{boxed::Box, sync::Arc};

use java_class_proto::MethodBody;
use jvm::{JavaError, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_core_arm::ArmCore;
use wie_wipi_java::WIPIJavaContextBase;

#[derive(Clone)]
pub struct KtfWIPIJavaContext {
    core: ArmCore,
    system: System,
    jvm: Arc<Jvm>,
}

impl KtfWIPIJavaContext {
    pub fn new(core: &ArmCore, system: &System, jvm: Arc<Jvm>) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
            jvm,
        }
    }
}

#[async_trait::async_trait]
impl WIPIJavaContextBase for KtfWIPIJavaContext {
    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: System,
            jvm: Arc<Jvm>,
            callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, JavaError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                let mut context = KtfWIPIJavaContext::new(&self.core, &self.system, self.jvm.clone());

                let _ = self.callback.call(&self.jvm, &mut context, Box::new([])).await?;

                Ok(0) // TODO resturn value
            }
        }

        self.system.clone().spawn(SpawnProxy {
            core: self.core.clone(),
            system: self.system.clone(),
            jvm: self.jvm.clone(),
            callback,
        });

        Ok(())
    }
}
