use alloc::{boxed::Box, sync::Arc, vec};
use core::time::Duration;

use wie_backend::{AsyncCallable, System};
use wie_core_arm::ArmCore;

use java_runtime::{File, FileStat, IOError, Runtime};
use jvm::{JavaError, Jvm, JvmCallback};

#[derive(Clone)]
pub struct KtfRuntime {
    core: ArmCore,
    system: System,
    jvm: Arc<Jvm>,
}

impl KtfRuntime {
    pub fn new(core: &mut ArmCore, system: &mut System, jvm: Arc<Jvm>) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
            jvm,
        }
    }
}

#[async_trait::async_trait]
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

        self.core.clone().spawn(SpawnProxy {
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
