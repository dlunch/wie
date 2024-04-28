use alloc::boxed::Box;
use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use wie_backend::{AsyncCallable, AsyncCallableResult};

use crate::{context::ArmCoreContext, Allocator, ArmCore};

pub struct SpawnFuture<C, R> {
    core: ArmCore,
    context: ArmCoreContext,
    stack_base: u32,
    callable_fut: Pin<Box<dyn Future<Output = R> + Send>>,
    _phantom: PhantomData<C>,
}

impl<C, R> SpawnFuture<C, R>
where
    C: AsyncCallable<R> + 'static + Send,
    R: AsyncCallableResult + 'static,
{
    pub fn new(mut core: ArmCore, callable: C) -> Self {
        let stack_base = Allocator::alloc(&mut core, 0x1000).unwrap();
        let context = ArmCoreContext::new(stack_base);
        let callable_fut = Box::pin(callable.call());

        Self {
            core,
            context,
            stack_base,
            callable_fut,
            _phantom: PhantomData,
        }
    }
}

impl<C, R> Future for SpawnFuture<C, R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.core.clone().restore_context(&self.context); // XXX clone is added to satisfy borrow checker
        let result = self.callable_fut.as_mut().poll(cx);
        self.context = self.core.save_context();

        if let Poll::Ready(x) = result {
            let stack_base = self.stack_base;
            Allocator::free(&mut self.core, stack_base).unwrap();

            Poll::Ready(x)
        } else {
            Poll::Pending
        }
    }
}

impl<C, R> Unpin for SpawnFuture<C, R> {}
