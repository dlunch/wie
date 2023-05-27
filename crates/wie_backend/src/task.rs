use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{time::Instant, CoreExecutor};

pub fn sleep(until: Instant) -> SleepFuture {
    SleepFuture::new(until)
}

pub fn yield_now() -> YieldFuture {
    YieldFuture {}
}

pub fn spawn<F, Fut, R>(f: F)
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = R> + 'static,
{
    let mut executor = CoreExecutor::current();
    executor.spawn(f);
}

pub struct YieldFuture {}

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

pub struct SleepFuture {
    until: Instant,
    registered: bool,
}

impl SleepFuture {
    pub fn new(until: Instant) -> Self {
        Self { until, registered: false }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            let task_id = CoreExecutor::current_task_id().unwrap();
            CoreExecutor::current().sleep(task_id, self.until);

            self.registered = true;

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Unpin for SleepFuture {}
