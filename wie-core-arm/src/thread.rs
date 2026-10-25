use alloc::sync::Arc;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_util::{ByteRead, ByteWrite, Result, write_generic};

use crate::{Allocator, ArmCore, context::ArmCoreContext, engine::DebugInner};

pub const STACK_SIZE: u32 = 0x100000; // 1 MB

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ThreadControl {
    pub context: ArmCoreContext,
    pub saved_contexts: u32,
    pub runtime_context: u32,
}

pub struct ThreadState {
    core: ArmCore,
    debug: Option<Arc<DebugInner>>,
    pub control_address: u32,
}

impl ThreadState {
    pub fn new(mut core: ArmCore) -> Result<Self> {
        let control_address = Allocator::alloc(&mut core, size_of::<ThreadControl>() as u32 + STACK_SIZE)?;
        let mut control = ThreadControl::zeroed();
        control.context.sp = control_address + size_of::<ThreadControl>() as u32 + STACK_SIZE;
        control.context.cpsr = 0x10; // USR32
        if let Err(error) = write_generic(&mut core, control_address, control) {
            Allocator::free(&mut core, control_address, size_of::<ThreadControl>() as u32 + STACK_SIZE)?;
            return Err(error);
        }

        Ok(Self {
            debug: core.debug_inner(),
            core,
            control_address,
        })
    }
}

// The debugger must access suspended guest state while the engine is stopped under its outer lock.
impl ByteRead for ThreadState {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize> {
        match &self.debug {
            Some(debug) => debug.read_memory(address, result),
            None => self.core.read_bytes(address, result),
        }
    }
}

impl ByteWrite for ThreadState {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> Result<()> {
        match &self.debug {
            Some(debug) => debug.write_memory(address, data),
            None => self.core.write_bytes(address, data),
        }
    }
}

impl Drop for ThreadState {
    fn drop(&mut self) {
        if let Err(err) = Allocator::free(&mut self.core, self.control_address, size_of::<ThreadControl>() as u32 + STACK_SIZE) {
            tracing::error!("Failed to free thread stack: {err}");
        }
    }
}
