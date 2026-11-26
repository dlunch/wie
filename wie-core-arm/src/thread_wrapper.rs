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
        let _guard = self.core.enter_thread_context(self.thread_id);

        if let Some(debug) = self.core.debug_inner()
            && !debug.is_thread_resumed(self.thread_id)
        {
            return Poll::Pending;
        }

        self.future.as_mut().poll(cx)
    }
}

impl Unpin for ArmCoreThreadWrapper {}

impl Drop for ArmCoreThreadWrapper {
    fn drop(&mut self) {
        self.core.delete_thread_context(self.thread_id);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use alloc::{sync::Arc, vec, vec::Vec};
    use core::{
        pin::pin,
        sync::atomic::{AtomicU32, Ordering},
    };
    use wie_backend::Options;

    use crate::engine::DebuggedArm32CpuEngine;

    use super::*;

    #[test]
    fn scheduler_locking_keeps_other_thread_futures_stopped() {
        let mut core = ArmCore::new(Options::default()).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let mut task = pin!(
            core.run_in_thread(move || async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .unwrap()
        );
        let _other = core.run_in_thread(|| async { Ok(()) }).unwrap();
        debug.resume(Vec::new(), Some(vec![2]));
        let (waker, wake_count) = futures_test::task::new_count_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(wake_count, 0);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        debug.resume(Vec::new(), None);
        assert!(matches!(task.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert_eq!(observed.load(Ordering::Relaxed), 1);
    }
}
