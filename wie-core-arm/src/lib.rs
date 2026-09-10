#![no_std]
extern crate alloc;

mod allocator;
mod aot;
mod context;
mod core;
mod engine;
mod function;
mod gdb;
mod thread;
mod thread_wrapper;

pub type ThreadId = usize;

pub use self::{
    allocator::Allocator,
    context::ArmCoreContext,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    function::{EmulatedFunction, EmulatedFunctionParam, JumpTo, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId},
};
