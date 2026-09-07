use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use wie_util::Result;

#[async_trait::async_trait]
pub trait TaskRunner: Sync + Send {
    /// Runs before scheduling tasks, including when every task is sleeping.
    fn before_tick(&self) {}

    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()>;
}

pub struct DefaultTaskRunner;

#[async_trait::async_trait]
impl TaskRunner for DefaultTaskRunner {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        future.await
    }
}
