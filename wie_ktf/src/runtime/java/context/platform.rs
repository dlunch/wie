use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::time::Duration;
use wie_core_arm::ArmCore;

use java_runtime::Runtime;
use jvm::JvmCallback;

use wie_backend::{AsyncCallable, SystemHandle};

use crate::runtime::KtfJavaContext;

#[derive(Clone)]
pub struct KtfRuntime {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfRuntime {
    pub fn new(core: ArmCore, system: SystemHandle) -> Self {
        Self { core, system }
    }
}

#[async_trait::async_trait(?Send)]
impl Runtime for KtfRuntime {
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
            core: ArmCore,
            system: SystemHandle,
            callback: Box<dyn JvmCallback>,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                let context = KtfJavaContext::new(&mut self.core, &mut self.system);
                let mut jvm = context.jvm();

                self.callback.call(&mut jvm, vec![].into_boxed_slice()).await?;

                Ok(0) // TODO
            }
        }

        self.core.clone().spawn(SpawnProxy {
            // TODO remove clone
            core: self.core.clone(),
            system: self.system.clone(),
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

    fn load_resource(&self, name: &str) -> Option<Vec<u8>> {
        let resource_id = self.system.resource().id(name);

        resource_id.map(|x| self.system.resource().data(x).to_vec())
    }

    fn println(&self, s: &str) {
        tracing::info!("println {}", s);
    }
}
