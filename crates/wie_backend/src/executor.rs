use std::{
    cell::{RefCell, RefMut},
    future::Future,
    mem::swap,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use wie_base::Core;

thread_local! {
    #[allow(clippy::type_complexity)]
    pub static EXECUTOR_INNER: RefCell<Option<Rc<RefCell<ExecutorInner>>>> = RefCell::new(None);
}

pub struct ExecutorInner {
    core: Box<dyn Core>,
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

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
            tasks: Vec::new(),
        };

        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn spawn<R>(&mut self, future: impl Future<Output = R> + 'static) {
        let fut = async move {
            future.await;
        };

        self.inner.borrow_mut().tasks.push(Box::pin(fut));
    }

    pub fn run(mut self) {
        loop {
            self.tick();

            if self.inner.borrow_mut().tasks.is_empty() {
                break;
            }
        }
    }

    pub fn current_executor() -> CoreExecutor {
        EXECUTOR_INNER.with(|f| {
            let inner = f.borrow().as_ref().unwrap().clone();
            Self { inner }
        })
    }

    pub fn core_mut(&self) -> RefMut<'_, Box<dyn Core>> {
        RefMut::map(self.inner.borrow_mut(), |x| &mut x.core)
    }

    fn tick(&mut self) {
        let waker = Self::dummy_waker();
        let mut context = Context::from_waker(&waker);

        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().replace(self.inner.clone());
        });

        let mut next_tasks = Vec::new();
        let mut tasks = Vec::new();
        swap(&mut tasks, &mut self.inner.borrow_mut().tasks);

        for mut task in tasks.drain(..) {
            match task.as_mut().poll(&mut context) {
                Poll::Ready(_) => {}
                Poll::Pending => next_tasks.push(task),
            }
        }
        EXECUTOR_INNER.with(|f| {
            f.borrow_mut().take();
        });

        self.inner.borrow_mut().tasks = next_tasks;
    }

    fn dummy_raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            CoreExecutor::dummy_raw_waker()
        }

        let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(core::ptr::null::<()>(), vtable)
    }

    fn dummy_waker() -> Waker {
        unsafe { Waker::from_raw(Self::dummy_raw_waker()) }
    }
}
