use alloc::boxed::Box;

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

use crate::JvmSupport;

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

        impl AsyncCallable<Result<(), WieError>> for SpawnProxy {
            async fn call(self) -> Result<(), WieError> {
                let mut context = WieJvmContext { system: self.system };

                let result = self.callback.call(&self.jvm, &mut context, Box::new([])).await;
                if let Err(err) = result {
                    return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
                }

                Ok(())
            }
        }

        let system = self.system();

        system.spawn(SpawnProxy {
            jvm: jvm.clone(),
            system: system.clone(),
            callback,
        });

        Ok(())
    }
}

pub type WieJavaClassProto = JavaClassProto<WieJvmContext>;
