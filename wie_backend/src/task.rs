use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{executor::Executor, time::Instant};

pub fn sleep(until: Instant) -> SleepFuture {
    SleepFuture::new(until)
}

pub fn yield_now() -> YieldFuture {
    YieldFuture {}
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
            Executor::current().sleep(self.until);

            self.registered = true;

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Unpin for SleepFuture {}
