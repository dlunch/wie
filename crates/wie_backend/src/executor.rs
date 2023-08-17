use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures::task::noop_waker;

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

    pub fn spawn<C, R, E>(&mut self, callable: C) -> usize
    where
        C: AsyncCallable<R, E> + 'static,
        E: Debug,
    {
        let context = self.inner.borrow_mut().module.core_mut().new_context();

        let context1 = context.clone();
        let fut = async move {
            callable.call().await.map_err(|x| anyhow::anyhow!("{:?}", x))?;
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
        let _guard = ExecutorGuard::new(self.inner.clone());

        let start = time.now();
        loop {
            let now = time.now();

            // TODO hardcode
            if now - start > 8 {
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

        Ok(())
    }

    pub fn run<C, R, E>(&mut self, time: &Time, callable: C) -> anyhow::Result<()>
    where
        C: AsyncCallable<R, E> + 'static,
        E: Debug,
    {
        let _guard = ExecutorGuard::new(self.inner.clone());

        let task_id = self.spawn(callable);

        loop {
            let now = time.now();
            self.step(now)?;

            if !self.inner.borrow().tasks.contains_key(&task_id) {
                break;
            }
        }

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

            let waker = noop_waker();
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

        self.inner.borrow_mut().sleeping_tasks.extend(sleeping_tasks);
        self.inner.borrow_mut().tasks.extend(next_tasks);

        Ok(())
    }

    pub(crate) fn sleep(&mut self, task_id: usize, until: Instant) {
        self.inner.borrow_mut().sleeping_tasks.insert(task_id, until);
    }
}

struct Task {
    fut: Pin<Box<dyn Future<Output = anyhow::Result<()>>>>,
    context: Box<dyn CoreContext>,
}

struct ExecutorGuard {}

impl ExecutorGuard {
    pub fn new(inner: Rc<RefCell<ExecutorInner>>) -> Self {
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(inner);
        });

        Self {}
    }
}

impl Drop for ExecutorGuard {
    fn drop(&mut self) {
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().take();
        });
    }
}
