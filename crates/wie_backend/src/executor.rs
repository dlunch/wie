use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::{pin, Pin},
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread::sleep,
    time::Duration,
};

use crate::time::{Instant, Time};

thread_local! {
    #[allow(clippy::type_complexity)]
    pub static EXECUTOR_INNER: RefCell<Option<Rc<RefCell<ExecutorInner>>>> = RefCell::new(None);
}

pub struct ExecutorInner {
    current_task_id: Option<usize>,
    tasks: HashMap<usize, Task>,
    sleeping_tasks: HashMap<usize, Instant>,
    last_task_id: usize,
}

#[async_trait::async_trait(?Send)]
pub trait AsyncCallable<R, E> {
    async fn call(self) -> Result<R, E>;
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, Fut> AsyncCallable<R, E> for F
where
    F: FnOnce() -> Fut + 'static,
    E: Debug,
    Fut: Future<Output = Result<R, E>> + 'static,
{
    async fn call(self) -> Result<R, E> {
        self().await
    }
}

// We abuse rust async to implement generator.
// CoreExecutor polls future even it's pending state, to make generator future to be able to continue.
pub struct Executor {
    inner: Rc<RefCell<ExecutorInner>>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        let inner = Rc::new(RefCell::new(ExecutorInner {
            current_task_id: None,
            tasks: HashMap::new(),
            sleeping_tasks: HashMap::new(),
            last_task_id: 0,
        }));

        let inner1 = inner.clone();

        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(inner1);
        });

        Self { inner }
    }

    pub fn spawn<C, R, E>(&mut self, callable: C) -> usize
    where
        C: AsyncCallable<R, E> + 'static,
        E: Debug,
    {
        let fut = async move {
            callable.call().await.map_err(|x| anyhow::anyhow!("{:?}", x))?;

            Ok::<(), anyhow::Error>(())
        };

        let task_id = {
            let mut inner = self.inner.borrow_mut();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.borrow_mut().tasks.insert(task_id, Task { fut: Box::pin(fut) });

        task_id
    }

    pub fn current() -> Executor {
        EXECUTOR_INNER.with(|f| {
            let inner = f.borrow().as_ref().unwrap().clone();
            Self { inner }
        })
    }

    pub fn tick(&mut self, time: &Time) -> anyhow::Result<()> {
        let end = time.now() + 8; // TODO hardcoded
        loop {
            let now = time.now();

            // TODO hardcode
            if now > end {
                break;
            }

            let running_task_count = self.inner.borrow().tasks.len() - self.inner.borrow().sleeping_tasks.len();
            if running_task_count == 0 && !self.inner.borrow().sleeping_tasks.is_empty() {
                let next_wakeup = *self.inner.borrow().sleeping_tasks.values().min().unwrap();
                if now < next_wakeup {
                    break;
                }
            }

            self.step(now)?;
        }

        let now = time.now();
        if now < end {
            sleep(Duration::from_millis(end - now));
        }

        Ok(())
    }

    pub fn run<F>(&mut self, future: F) -> F::Output
    where
        F: Future,
    {
        let waker = self.create_waker();
        let mut context = Context::from_waker(&waker);

        let mut future = pin!(future);

        loop {
            if let Poll::Ready(x) = future.as_mut().poll(&mut context) {
                return x;
            }
        }
    }

    fn step(&mut self, now: Instant) -> anyhow::Result<()> {
        let mut next_tasks = HashMap::new();
        let tasks = self.inner.borrow_mut().tasks.drain().collect::<HashMap<_, _>>();
        let mut sleeping_tasks = self.inner.borrow_mut().sleeping_tasks.drain().collect::<HashMap<_, _>>();

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
            self.inner.borrow_mut().current_task_id = Some(task_id);

            match task.fut.as_mut().poll(&mut context) {
                Poll::Ready(x) => {
                    x?;
                }
                Poll::Pending => {
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.borrow_mut().current_task_id = None;
        }

        self.inner.borrow_mut().sleeping_tasks.extend(sleeping_tasks);
        self.inner.borrow_mut().tasks.extend(next_tasks);

        Ok(())
    }

    pub(crate) fn sleep(&mut self, until: Instant) {
        let task_id = self.inner.borrow().current_task_id.unwrap();

        self.inner.borrow_mut().sleeping_tasks.insert(task_id, until);
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

struct Task {
    fut: Pin<Box<dyn Future<Output = anyhow::Result<()>>>>,
}
