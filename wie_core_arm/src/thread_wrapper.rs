use alloc::boxed::Box;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use wie_util::Result;

use crate::{Allocator, ArmCore, context::ArmCoreContext};

const STACK_SIZE: u32 = 0x1000;

pub struct ArmCoreThreadWrapper {
    core: ArmCore,
    context: ArmCoreContext,
    stack_base: u32,
    future: Pin<Box<dyn Future<Output = Result<()>> + Send>>,
}

impl ArmCoreThreadWrapper {
    pub fn new<F, Fut>(mut core: ArmCore, entry: F) -> Result<Self>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let stack_base = Allocator::alloc(&mut core, STACK_SIZE)?;
        let context = ArmCoreContext {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            sb: 0,
            sl: 0,
            fp: 0,
            ip: 0,
            sp: stack_base + STACK_SIZE,
            lr: 0,
            pc: 0,
            cpsr: 0x10, // USR32
        };

        Ok(Self {
            core,
            context,
            stack_base,
            future: Box::pin(entry()),
        })
    }
}

impl Drop for ArmCoreThreadWrapper {
    fn drop(&mut self) {
        Allocator::free(&mut self.core, self.stack_base, STACK_SIZE).unwrap()
    }
}

impl Future for ArmCoreThreadWrapper {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let context = self.context.clone(); // for borrow checker
        self.core.restore_context(&context);

        let result = self.future.as_mut().poll(cx);

        self.context = self.core.save_context();

        result
    }
}

impl Unpin for ArmCoreThreadWrapper {}
