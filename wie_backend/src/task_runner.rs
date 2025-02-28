use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use wie_util::Result;

#[async_trait::async_trait]
pub trait TaskRunner: Sync + Send {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>);
}

pub struct DefaultTaskRunner;

#[async_trait::async_trait]
impl TaskRunner for DefaultTaskRunner {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) {
        future.await.unwrap();
    }
}
