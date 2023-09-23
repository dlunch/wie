use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::LocalBoxFuture, FutureExt};

use wie_backend::AsyncCallable;
use wie_base::util::ByteWrite;

use crate::{
    context::ArmCoreContext,
    core::{ArmCoreResult, RUN_FUNCTION_LR},
    Allocator, ArmCore,
};

pub struct RunFunctionFuture<R> {
    core: ArmCore,
    waiting_fut: Option<LocalBoxFuture<'static, ArmCoreResult<()>>>,
    previous_context: ArmCoreContext,
    _phantom: PhantomData<R>,
}

impl<R> RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    pub fn new(core: &mut ArmCore, address: u32, params: &[u32]) -> Self {
        let previous_context = core.save_context();

        let mut context = previous_context.clone();
        Self::set_context(core, &mut context, address, params);
        core.restore_context(&context);

        Self {
            core: core.clone(),
            waiting_fut: None,
            previous_context,
            _phantom: PhantomData,
        }
    }

    pub fn set_context(core: &mut ArmCore, context: &mut ArmCoreContext, address: u32, params: &[u32]) {
        // is there cleaner way to do this?
        if !params.is_empty() {
            context.r0 = params[0];
        }
        if params.len() > 1 {
            context.r1 = params[1];
        }
        if params.len() > 2 {
            context.r2 = params[2];
        }
        if params.len() > 3 {
            context.r3 = params[3];
        }
        if params.len() > 4 {
            for param in params[4..].iter() {
                context.sp -= 4;
                core.write_bytes(context.sp, &param.to_le_bytes()).unwrap();
            }
        }

        context.pc = address;
        context.lr = RUN_FUNCTION_LR;
    }
}

impl<R> Future for RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    type Output = ArmCoreResult<R>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(fut) = &mut self.waiting_fut {
            let poll = fut.as_mut().poll(cx);

            if let Poll::Ready(x) = poll {
                if x.is_err() {
                    return Poll::Ready(Err(x.err().unwrap()));
                }

                self.waiting_fut = None;
            } else {
                return Poll::Pending;
            }
        }

        let (pc, _) = self.core.read_pc_lr().unwrap();

        if pc == RUN_FUNCTION_LR {
            let result = R::get(&self.core);

            let previous_context = self.previous_context.clone();
            self.core.restore_context(&previous_context);

            Poll::Ready(Ok(result))
        } else {
            let fut = self.core.clone().run();
            self.waiting_fut = Some(fut.boxed_local());

            Poll::Pending
        }
    }
}

impl<R> Unpin for RunFunctionFuture<R> where R: RunFunctionResult<R> {}

pub trait RunFunctionResult<R> {
    fn get(core: &ArmCore) -> R;
}

impl RunFunctionResult<u32> for u32 {
    fn get(core: &ArmCore) -> u32 {
        core.read_param(0).unwrap()
    }
}

impl RunFunctionResult<()> for () {
    fn get(_: &ArmCore) {}
}

pub struct SpawnFuture<C, R, E> {
    core: ArmCore,
    context: ArmCoreContext,
    stack_base: u32,
    callable_fut: LocalBoxFuture<'static, Result<R, E>>,
    _phantom: PhantomData<C>,
}

impl<C, R, E> SpawnFuture<C, R, E>
where
    C: AsyncCallable<R, E> + 'static,
    R: 'static,
    E: core::fmt::Debug + 'static,
{
    pub fn new(mut core: ArmCore, callable: C) -> Self {
        let stack_base = Allocator::alloc(&mut core, 0x1000).unwrap();
        let context = ArmCoreContext::new(stack_base);
        let callable_fut = callable.call().boxed_local();

        Self {
            core,
            context,
            stack_base,
            callable_fut,
            _phantom: PhantomData,
        }
    }
}

impl<C, R, E> Future for SpawnFuture<C, R, E> {
    type Output = Result<R, E>;

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

impl<C, R, E> Unpin for SpawnFuture<C, R, E> {}
