use wie_util::Result;

use crate::{Allocator, ArmCore, context::ArmCoreContext};

const STACK_SIZE: u32 = 0x1000;

pub struct ThreadState {
    core: ArmCore,
    pub context: ArmCoreContext,
    pub stack_base: usize,
    pub stack_size: usize,
}

impl ThreadState {
    pub fn new(mut core: ArmCore) -> Result<Self> {
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
            stack_base: stack_base as _,
            stack_size: STACK_SIZE as _,
        })
    }
}

impl Drop for ThreadState {
    fn drop(&mut self) {
        Allocator::free(&mut self.core, self.stack_base as _, self.stack_size as u32).unwrap()
    }
}
