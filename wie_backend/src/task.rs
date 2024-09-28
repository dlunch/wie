use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{executor::Executor, time::Instant};

pub struct YieldFuture;

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

pub struct SleepFuture {
    polled: bool,
}

impl SleepFuture {
    pub fn new(until: Instant, executor: &mut Executor) -> Self {
        executor.sleep(until);

        Self { polled: false }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.polled {
            self.polled = true;

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Unpin for SleepFuture {}
