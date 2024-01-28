use alloc::{boxed::Box, string::String, vec::Vec};
use core::time::Duration;

use java_runtime::Runtime;
use jvm::JvmCallback;

#[derive(Clone)]
pub struct DummyRuntime;

#[async_trait::async_trait(?Send)]
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
