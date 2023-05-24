use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use crate::CoreExecutor;

pub fn sleep(duration: u64) -> SleepFuture {
    SleepFuture::new(duration)
}

pub fn yield_now() -> YieldFuture {
    YieldFuture {}
}

pub fn spawn<R>(future: impl Future<Output = R> + 'static) {
    let mut executor = CoreExecutor::current_executor();
    executor.spawn(future);
}

pub struct YieldFuture {}

impl Future for YieldFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

pub struct SleepFuture {
    duration: u64,
    state: Arc<Mutex<bool>>,
    waker: Option<Waker>,
}

impl SleepFuture {
    pub fn new(duration: u64) -> Self {
        Self {
            duration,
            state: Arc::new(Mutex::new(false)),
            waker: None,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.waker.is_none() {
            self.waker = Some(cx.waker().clone());

            // TODO temp
            let waker = cx.waker().clone();
            let duration = Duration::from_millis(self.duration);
            let state1 = self.state.clone();
            thread::spawn(move || {
                thread::sleep(duration);
                *state1.lock().unwrap() = true;
                waker.wake_by_ref();
            });
        }

        if *self.state.lock().unwrap() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Unpin for SleepFuture {}
