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
    ArmCore,
};

pub struct RunFunctionFuture<R> {
    previous_context: ArmCoreContext,
    context: Option<ArmCoreContext>,
    waiting_fut: Option<LocalBoxFuture<'static, ArmCoreResult<ArmCoreContext>>>,
    _phantom: PhantomData<R>,
}

impl<R> RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    pub fn from_context(context: ArmCoreContext, previous_context: ArmCoreContext) -> Self {
        Self {
            previous_context,
            context: Some(context),
            waiting_fut: None,
            _phantom: PhantomData,
        }
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
        let executor = CoreExecutor::current_executor();

        let mut core = executor.core_mut();
        let core = core.as_any_mut().downcast_mut::<ArmCore>().unwrap();

        if self.context.as_ref().unwrap().pc == RUN_FUNCTION_LR {
            let result = R::get(self.context.as_ref().unwrap());
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
