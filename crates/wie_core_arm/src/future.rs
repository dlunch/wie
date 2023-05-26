use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::LocalBoxFuture, FutureExt};
use wie_backend::CoreExecutor;

use crate::{
    context::ArmCoreContext,
    core::{ArmCoreResult, RUN_FUNCTION_LR},
    Allocator, ArmCore,
};

pub struct RunFunctionFuture<R> {
    previous_context: ArmCoreContext,
    context: Option<ArmCoreContext>,
    stack_base: u32,
    waiting_fut: Option<LocalBoxFuture<'static, ArmCoreResult<ArmCoreContext>>>,
    _phantom: PhantomData<R>,
}

impl<R> RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    pub fn new(core: &mut ArmCore, address: u32, params: &[u32]) -> Self {
        let previous_context = ArmCoreContext::from_uc(&core.uc);
        let mut context = previous_context.clone();

        let stack_base = Allocator::alloc(core, 0x1000).unwrap();
        Self::set_context(core, &mut context, stack_base + 0x1000, address, params);

        Self {
            previous_context,
            context: Some(context),
            stack_base,
            waiting_fut: None,
            _phantom: PhantomData,
        }
    }

    pub fn set_context(core: &mut ArmCore, context: &mut ArmCoreContext, sp: u32, address: u32, params: &[u32]) {
        context.sp = sp;

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
                core.uc.mem_write(context.sp as u64, &param.to_le_bytes()).unwrap();
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
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(fut) = &mut self.waiting_fut {
            let poll = fut.as_mut().poll(cx);

            if let Poll::Ready(x) = poll {
                self.waiting_fut = None;
                self.context = Some(x.unwrap());
            } else {
                return Poll::Pending;
            }
        }
        let executor = CoreExecutor::current();

        let mut core = executor.core_mut();
        let core = core.as_any_mut().downcast_mut::<ArmCore>().unwrap();

        if self.context.as_ref().unwrap().pc == RUN_FUNCTION_LR {
            let result = R::get(self.context.as_ref().unwrap());
            Allocator::free(core, self.stack_base).unwrap();

            core.restore_context(&self.previous_context).unwrap();

            Poll::Ready(result)
        } else {
            let core: &mut ArmCore = unsafe { core::mem::transmute(core) }; // TODO
            let fut = core.run(self.context.take().unwrap());
            self.waiting_fut = Some(fut.boxed_local());

            Poll::Pending
        }
    }
}

impl<R> Unpin for RunFunctionFuture<R> where R: RunFunctionResult<R> {}

pub trait RunFunctionResult<R> {
    fn get(context: &ArmCoreContext) -> R;
}

impl RunFunctionResult<u32> for u32 {
    fn get(context: &ArmCoreContext) -> u32 {
        context.r0
    }
}

impl RunFunctionResult<()> for () {
    fn get(_: &ArmCoreContext) {}
}
