use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use wie_base::{CoreContext, Module};

use crate::time::{Instant, Time};

thread_local! {
    #[allow(clippy::type_complexity)]
    pub static EXECUTOR_INNER: RefCell<Option<Rc<RefCell<ExecutorInner>>>> = RefCell::new(None);
}

pub struct ExecutorInner {
    module: Box<dyn Module>,
    current_task_id: Option<usize>,
    tasks: HashMap<usize, Task>,
    sleeping_tasks: HashMap<usize, Instant>,
    last_task_id: usize,
}

// We abuse rust async to implement generator.
// CoreExecutor polls future even it's pending state, to make generator future to be able to continue.
pub struct Executor {
    inner: Rc<RefCell<ExecutorInner>>,
}

impl Executor {
    pub(crate) fn new<M>(module: M) -> Self
    where
        M: Module + 'static,
    {
        let inner: ExecutorInner = ExecutorInner {
            module: Box::new(module),
            current_task_id: None,
            tasks: HashMap::new(),
            sleeping_tasks: HashMap::new(),
            last_task_id: 0,
        };

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn spawn<F, Fut, E, R>(&mut self, f: F) -> usize
    where
        F: FnOnce() -> Fut + 'static,
        E: Debug,
        Fut: Future<Output = Result<R, E>> + 'static,
    {
        let context = self.inner.borrow_mut().module.core_mut().new_context();

        let context1 = context.clone();
        let fut = async move {
            f().await.map_err(|x| anyhow::anyhow!("{:?}", x))?;
            Executor::current().module_mut().core_mut().free_context(context1);

            Ok::<(), anyhow::Error>(())
        };

        let task_id = {
            let mut inner = self.inner.borrow_mut();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.borrow_mut().tasks.insert(task_id, Task { fut: Box::pin(fut), context });

        task_id
    }

    pub fn current() -> Executor {
        EXECUTOR_INNER.with(|f| {
            let inner = f.borrow().as_ref().unwrap().clone();
            Self { inner }
        })
    }

    pub fn current_task_id() -> Option<usize> {
        EXECUTOR_INNER.with(|f| f.borrow().as_ref().unwrap().borrow().current_task_id)
    }

    pub fn module_mut(&self) -> RefMut<'_, Box<dyn Module>> {
        RefMut::map(self.inner.borrow_mut(), |x| &mut x.module)
    }

    pub fn tick(&mut self, time: &Time) -> anyhow::Result<()> {
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(self.inner.clone());
        });

        let start = time.now();
        loop {
            let now = time.now();

            // TODO hardcode
            if now - start > 8 {
                break;
            }

            self.step(now)?;
        }

        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().take();
        });

        Ok(())
    }

    pub fn run<F, Fut, E, R>(&mut self, time: &Time, f: F) -> anyhow::Result<()>
    where
        F: FnOnce() -> Fut + 'static,
        E: Debug,
        Fut: Future<Output = Result<R, E>> + 'static,
    {
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(self.inner.clone());
        });

        let task_id = self.spawn(f);

        loop {
            let now = time.now();
            self.step(now)?;

            if !self.inner.borrow().tasks.contains_key(&task_id) {
                break;
            }
        }

        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().take();
        });

        Ok(())
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

            let waker = Self::waker_from_task_id(task_id);
            let mut context = Context::from_waker(&waker);
            self.inner.borrow_mut().current_task_id = Some(task_id);

            self.inner.borrow_mut().module.core_mut().restore_context(&*task.context);
            match task.fut.as_mut().poll(&mut context) {
                Poll::Ready(x) => {
                    if x.is_err() {
                        return Err(x
                            .map_err(|x| {
                                let reg_stack = self.inner.borrow_mut().module.core_mut().dump_reg_stack();

                                anyhow::anyhow!("{}\n{}", x, reg_stack)
                            })
                            .unwrap_err());
                    }
                }
                Poll::Pending => {
                    task.context = self.inner.borrow_mut().module.core_mut().save_context();
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.borrow_mut().current_task_id = None;
        }

        self.inner.borrow_mut().sleeping_tasks.extend(sleeping_tasks.into_iter());
        self.inner.borrow_mut().tasks.extend(next_tasks.into_iter());

        Ok(())
    }

    pub(crate) fn sleep(&mut self, task_id: usize, until: Instant) {
        self.inner.borrow_mut().sleeping_tasks.insert(task_id, until);
    }

    fn dummy_raw_waker(task_id: usize) -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(data: *const ()) -> RawWaker {
            Executor::dummy_raw_waker(data as usize)
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
