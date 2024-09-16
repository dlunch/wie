use alloc::boxed::Box;

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

#[derive(Clone)]
pub struct WieJvmContext {
    system: System,
}

impl WieJvmContext {
    pub fn new(system: &System) -> Self {
        Self { system: system.clone() }
    }

    pub fn system(&mut self) -> &mut System {
        &mut self.system
    }

    pub fn spawn(&mut self, jvm: &Jvm, callback: Box<dyn MethodBody<JavaError, WieJvmContext>>) -> JvmResult<()> {
        struct SpawnProxy {
            jvm: Jvm,
            system: System,
            callback: Box<dyn MethodBody<JavaError, WieJvmContext>>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, WieError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, WieError> {
                let mut context = WieJvmContext { system: self.system };
                let _ = self.callback.call(&self.jvm, &mut context, Box::new([])).await?;

                Ok(0) // TODO return value
            }
        }

        self.system().clone().spawn(SpawnProxy {
            jvm: jvm.clone(),
            system: self.system.clone(),
            callback,
        });

        Ok(())
    }
}

pub type WieJavaClassProto = JavaClassProto<WieJvmContext>;
