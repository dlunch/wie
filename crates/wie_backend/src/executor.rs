use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use wie_base::{Core, CoreContext};

thread_local! {
    #[allow(clippy::type_complexity)]
    pub static CORE: RefCell<Option<Rc<RefCell<Box<dyn Core>>>>> = RefCell::new(None);
}

pub struct CoreExecutor {
    core: Rc<RefCell<Box<dyn Core>>>,
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl CoreExecutor {
    pub fn new<C>(core: C) -> Self
    where
        C: Core + 'static,
    {
        Self {
            core: Rc::new(RefCell::new(Box::new(core))),
            tasks: Vec::new(),
        }
    }

    pub fn spawn<R>(&mut self, future: impl Future<Output = R> + 'static) {
        let fut = async move {
            future.await;
        };

        self.tasks.push(Box::pin(fut));
    }

    pub fn tick(&mut self) {
        let waker = Self::dummy_waker();
        let mut context = Context::from_waker(&waker);

        CORE.with(|f| {
            f.borrow_mut().replace(self.core.clone());
        });

        let mut next_tasks = Vec::new();
        for mut task in self.tasks.drain(..) {
            match task.as_mut().poll(&mut context) {
                Poll::Ready(_) => {}
                Poll::Pending => next_tasks.push(task),
            }
        }
        CORE.with(|f| {
            f.borrow_mut().take();
        });

        self.tasks = next_tasks;
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

pub trait CoreExecutorFuture<C>: Unpin
where
    C: CoreContext + 'static,
{
    fn get_core(&self) -> Rc<RefCell<Box<dyn Core>>> {
        CORE.with(|f| f.borrow().as_ref().unwrap().clone())
    }
}
