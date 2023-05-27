use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::LocalBoxFuture, FutureExt};
use unicorn_engine::RegisterARM;

use wie_backend::CoreExecutor;
use wie_base::Core;

use crate::{
    context::ArmCoreContext,
    core::{ArmCoreResult, RUN_FUNCTION_LR},
    ArmCore,
};

pub struct RunFunctionFuture<R> {
    waiting_fut: Option<LocalBoxFuture<'static, ArmCoreResult<()>>>,
    previous_context: ArmCoreContext,
    _phantom: PhantomData<R>,
}

impl<R> RunFunctionFuture<R>
where
    R: RunFunctionResult<R>,
{
    pub fn new(core: &mut ArmCore, address: u32, params: &[u32]) -> Self {
        let previous_context = *core.save_context().into_any().downcast::<ArmCoreContext>().unwrap();

        let mut context = previous_context.clone();
        Self::set_context(core, &mut context, address, params);
        core.restore_context(&context);

        Self {
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
    type Output = ArmCoreResult<R>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let executor = CoreExecutor::current();

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

        let mut core = executor.core_mut();
        let core = core.as_any_mut().downcast_mut::<ArmCore>().unwrap();

        let pc = core.uc.reg_read(RegisterARM::PC).unwrap() as u32;

        if pc == RUN_FUNCTION_LR {
            let result = R::get(core);
            core.restore_context(&self.previous_context);

            Poll::Ready(Ok(result))
        } else {
            let core1: &mut ArmCore = unsafe { core::mem::transmute(core as &mut ArmCore) }; // TODO
            let fut = core1.run();
            self.waiting_fut = Some(fut.boxed_local());

            Poll::Pending
        }
    }
}

impl<R> Unpin for RunFunctionFuture<R> where R: RunFunctionResult<R> {}

pub trait RunFunctionResult<R> {
    fn get(context: &ArmCore) -> R;
}

impl RunFunctionResult<u32> for u32 {
    fn get(context: &ArmCore) -> u32 {
        context.uc.reg_read(RegisterARM::R0).unwrap() as u32
    }
}

impl RunFunctionResult<()> for () {
    fn get(_: &ArmCore) {}
}
