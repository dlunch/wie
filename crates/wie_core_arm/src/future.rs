use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::LocalBoxFuture, FutureExt};

use wie_backend::Executor;
use wie_base::{util::ByteWrite, Core};

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
        let executor = Executor::current();

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

        let mut module = executor.module_mut();
        let core = module.core_mut().as_any_mut().downcast_mut::<ArmCore>().unwrap();

        let (pc, _) = core.read_pc_lr().unwrap();

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
