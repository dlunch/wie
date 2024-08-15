use alloc::boxed::Box;

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};

#[derive(Clone)]
pub struct WIPIJavaContext {
    system: System,
}

impl WIPIJavaContext {
    pub fn new(system: &System) -> Self {
        Self { system: system.clone() }
    }

    pub fn system(&mut self) -> &mut System {
        &mut self.system
    }

    pub fn spawn(&mut self, jvm: &Jvm, callback: Box<dyn MethodBody<JavaError, WIPIJavaContext>>) -> JvmResult<()> {
        struct SpawnProxy {
            jvm: Jvm,
            system: System,
            callback: Box<dyn MethodBody<JavaError, WIPIJavaContext>>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, JavaError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                let mut context = WIPIJavaContext { system: self.system };
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

pub(crate) type WIPIJavaClassProto = JavaClassProto<WIPIJavaContext>;
