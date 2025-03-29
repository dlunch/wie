use alloc::boxed::Box;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use wie_util::Result;

use crate::{ArmCore, ThreadId};

pub struct ArmCoreThreadWrapper {
    core: ArmCore,
    thread_id: ThreadId,
    future: Pin<Box<dyn Future<Output = Result<()>> + Send>>,
}

impl ArmCoreThreadWrapper {
    pub fn new<F, Fut>(core: ArmCore, thread_id: ThreadId, entry: F) -> Result<Self>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Ok(Self {
            core,
            thread_id,
            future: Box::pin(entry()),
        })
    }
}

impl Future for ArmCoreThreadWrapper {
    type Output = Result<()>;

    #[tracing::instrument(name = "native thread", fields(id = self.thread_id), skip_all)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let _ = self.core.enter_thread_context(self.thread_id);

        self.future.as_mut().poll(cx)
    }
}

impl Unpin for ArmCoreThreadWrapper {}

impl Drop for ArmCoreThreadWrapper {
    fn drop(&mut self) {
        self.core.delete_thread_context(self.thread_id);
    }
}
