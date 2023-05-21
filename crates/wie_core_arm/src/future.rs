use core::{
    cell::RefMut,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use wie_backend::CoreExecutorFuture;

use crate::{context::ArmCoreContext, core::RUN_FUNCTION_LR, ArmCore};

pub struct RunFunctionFuture<R> {
    previous_context: ArmCoreContext,
    context: ArmCoreContext,
    _phantom: PhantomData<R>,
}

impl<R> RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    pub fn from_context(context: ArmCoreContext, previous_context: ArmCoreContext) -> Self {
        Self {
            previous_context,
            context,
            _phantom: PhantomData,
        }
    }
}

impl<R> CoreExecutorFuture<ArmCoreContext> for RunFunctionFuture<R> where R: RunFunctionResult<R> {}

impl<R> Future for RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let core = self.get_core().clone();
        let mut core = RefMut::map(core.borrow_mut(), |x| (*x).as_any_mut().downcast_mut::<ArmCore>().unwrap());

        self.context = core.run(&self.context).unwrap();

        if self.context.lr == RUN_FUNCTION_LR {
            let result = R::get(&self.context);
            core.restore_context(&self.previous_context).unwrap();

            Poll::Ready(result)
        } else {
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
