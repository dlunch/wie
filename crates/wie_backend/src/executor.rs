use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use wie_base::{Core, CoreContext};

use crate::{time::Instant, Backend};

thread_local! {
    #[allow(clippy::type_complexity)]
    pub static EXECUTOR_INNER: RefCell<Option<Rc<RefCell<ExecutorInner>>>> = RefCell::new(None);
}

pub struct ExecutorInner {
    core: Box<dyn Core>,
    current_task_id: Option<usize>,
    tasks: HashMap<usize, Task>,
    sleeping_tasks: HashMap<usize, Instant>,
    last_task_id: usize,
}

// We abuse rust async to implement generator.
// CoreExecutor polls future even it's pending state, to make generator future to be able to continue.
pub struct CoreExecutor {
    inner: Rc<RefCell<ExecutorInner>>,
}

impl CoreExecutor {
    pub fn new<C>(core: C) -> Self
    where
        C: Core + 'static,
    {
        let inner = ExecutorInner {
            core: Box::new(core),
            current_task_id: None,
            tasks: HashMap::new(),
            sleeping_tasks: HashMap::new(),
            last_task_id: 0,
        };

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn spawn<F, Fut, E, R>(&mut self, f: F)
    where
        F: Fn() -> Fut + 'static,
        E: Debug,
        Fut: Future<Output = Result<R, E>> + 'static,
    {
        let context = self.inner.borrow_mut().core.new_context();

        let fut = async move {
            f().await.map_err(|x| anyhow::anyhow!("{:?}", x))?;
            // CoreExecutor::current().core_mut().free_context(context); // TODO

            Ok::<(), anyhow::Error>(())
        };

        let task_id = {
            let mut inner = self.inner.borrow_mut();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.borrow_mut().tasks.insert(task_id, Task { fut: Box::pin(fut), context });
    }

    pub fn run(mut self, backend: Backend) -> anyhow::Result<()> {
        loop {
            let now = backend.time().now();

            self.tick(now)?;

            if self.inner.borrow_mut().tasks.is_empty() {
                break;
            }
        }

        Ok(())
    }

    pub fn current() -> CoreExecutor {
        EXECUTOR_INNER.with(|f| {
            let inner = f.borrow().as_ref().unwrap().clone();
            Self { inner }
        })
    }

    pub fn current_task_id() -> Option<usize> {
        EXECUTOR_INNER.with(|f| f.borrow().as_ref().unwrap().borrow().current_task_id)
    }

    pub fn core_mut(&self) -> RefMut<'_, Box<dyn Core>> {
        RefMut::map(self.inner.borrow_mut(), |x| &mut x.core)
    }

    pub(crate) fn sleep(&mut self, task_id: usize, until: Instant) {
        self.inner.borrow_mut().sleeping_tasks.insert(task_id, until);
    }

    pub(crate) fn tick(&mut self, now: Instant) -> anyhow::Result<()> {
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(self.inner.clone());
        });

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

            let waker = Self::waker_from_task_id(task_id);
            let mut context = Context::from_waker(&waker);
            self.inner.borrow_mut().current_task_id = Some(task_id);

            self.inner.borrow_mut().core.restore_context(&*task.context);
            match task.fut.as_mut().poll(&mut context) {
                Poll::Ready(x) => {
                    if x.is_err() {
                        return Err(x.err().unwrap());
                    }
                }
                Poll::Pending => {
                    task.context = self.inner.borrow_mut().core.save_context();
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.borrow_mut().current_task_id = None;
        }
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().take();
        });

        self.inner.borrow_mut().sleeping_tasks.extend(sleeping_tasks.into_iter());
        self.inner.borrow_mut().tasks.extend(next_tasks.into_iter());

        Ok(())
    }

    fn dummy_raw_waker(task_id: usize) -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(data: *const ()) -> RawWaker {
            CoreExecutor::dummy_raw_waker(data as usize)
        }

        let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(task_id as _, vtable)
    }

    fn waker_from_task_id(task_id: usize) -> Waker {
        unsafe { Waker::from_raw(Self::dummy_raw_waker(task_id)) }
    }
}

struct Task {
    fut: Pin<Box<dyn Future<Output = anyhow::Result<()>>>>,
    context: Box<dyn CoreContext>,
}
