use alloc::sync::Arc;
use core::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use std::{collections::HashMap, sync::Mutex};

use crate::time::Instant;

type Task = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub struct ExecutorInner {
    current_task_id: Option<usize>,
    tasks: HashMap<usize, Task>,
    sleeping_tasks: HashMap<usize, Instant>,
    last_task_id: usize,
}

#[async_trait::async_trait]
pub trait AsyncCallable<R> {
    async fn call(self) -> R;
}

#[async_trait::async_trait]
impl<F, R, Fut> AsyncCallable<R> for F
where
    F: FnOnce() -> Fut + 'static + Send,
    R: AsyncCallableResult,
    Fut: Future<Output = R> + 'static + Send,
{
    async fn call(self) -> R {
        self().await
    }
}

pub trait AsyncCallableResult {
    fn err(&self) -> Option<anyhow::Error>;
}

impl<R, E> AsyncCallableResult for Result<R, E>
where
    E: Debug,
{
    fn err(&self) -> Option<anyhow::Error> {
        match self {
            Ok(_) => None,
            Err(e) => Some(anyhow::anyhow!("{:?}", e)),
        }
    }
}

impl AsyncCallableResult for () {
    fn err(&self) -> Option<anyhow::Error> {
        None
    }
}

// Executor polling every future until it is ready to implement generator using async ecosystem
#[derive(Clone)]
pub struct Executor {
    inner: Arc<Mutex<ExecutorInner>>,
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(ExecutorInner {
            current_task_id: None,
            tasks: HashMap::new(),
            sleeping_tasks: HashMap::new(),
            last_task_id: 0,
        }));

        Self { inner }
    }

    pub fn spawn<C, R>(&mut self, callable: C) -> usize
    where
        C: AsyncCallable<R> + 'static + Send,
        R: AsyncCallableResult,
    {
        let fut = async move {
            let result = callable.call().await;
            if let Some(err) = result.err() {
                return Err(err);
            }

            anyhow::Ok(())
        };

        let task_id = {
            let mut inner = self.inner.lock().unwrap();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.lock().unwrap().tasks.insert(task_id, Box::pin(fut));

        task_id
    }

    // TODO we need to remove error handling from here. we need to JoinHandle like on spawn..
    pub fn tick<T>(&mut self, now: T) -> anyhow::Result<()>
    where
        T: Fn() -> Instant,
    {
        let end = now() + 8; // TODO hardcoded
        loop {
            let now = now();

            if now > end {
                break;
            }

            {
                let inner = self.inner.lock().unwrap();
                let running_task_count = inner.tasks.len() - inner.sleeping_tasks.len();
                if running_task_count == 0 && !inner.sleeping_tasks.is_empty() {
                    let next_wakeup = *inner.sleeping_tasks.values().min().unwrap();
                    if now < next_wakeup {
                        break;
                    }
                }
            }

            self.step(now)?;
        }

        Ok(())
    }

    fn step(&mut self, now: Instant) -> anyhow::Result<()> {
        let mut next_tasks = HashMap::new();
        let tasks = self.inner.lock().unwrap().tasks.drain().collect::<HashMap<_, _>>();
        let mut sleeping_tasks = self.inner.lock().unwrap().sleeping_tasks.drain().collect::<HashMap<_, _>>();

        for (task_id, mut task) in tasks.into_iter() {
            let item = sleeping_tasks.get(&task_id);
            if let Some(item) = item {
                if *item <= now {
                    sleeping_tasks.remove(&task_id);
                } else {
                    next_tasks.insert(task_id, task);
                    continue;
                }
            }

            let waker = self.create_waker();
            let mut context = Context::from_waker(&waker);
            self.inner.lock().unwrap().current_task_id = Some(task_id);

            match task.as_mut().poll(&mut context) {
                Poll::Ready(x) => {
                    x?;
                }
                Poll::Pending => {
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.lock().unwrap().current_task_id = None;
        }

        self.inner.lock().unwrap().sleeping_tasks.extend(sleeping_tasks);
        self.inner.lock().unwrap().tasks.extend(next_tasks);

        Ok(())
    }

    pub(crate) fn sleep(&mut self, until: Instant) {
        let task_id = self.inner.lock().unwrap().current_task_id.unwrap();

        self.inner.lock().unwrap().sleeping_tasks.insert(task_id, until);
    }

    fn create_waker(&self) -> Waker {
        unsafe fn noop_clone(_data: *const ()) -> RawWaker {
            noop_raw_waker()
        }

        unsafe fn noop(_data: *const ()) {}

        const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

        const fn noop_raw_waker() -> RawWaker {
            RawWaker::new(core::ptr::null(), &NOOP_WAKER_VTABLE)
        }

        unsafe { Waker::from_raw(noop_raw_waker()) }
    }
}
