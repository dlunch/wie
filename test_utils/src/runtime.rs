use alloc::boxed::Box;
use core::time::Duration;

use java_runtime::{File, FileStat, IOError, Runtime};
use jvm::JvmCallback;

#[derive(Clone)]
pub struct DummyRuntime;

#[async_trait::async_trait]
impl Runtime for DummyRuntime {
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

    fn stdin(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
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
