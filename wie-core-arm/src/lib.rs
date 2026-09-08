#![no_std]
extern crate alloc;

mod allocator;
mod binary_patches;
mod context;
mod core;
mod engine;
mod function;
mod gdb;
pub mod stdlib;
mod thread;
mod thread_wrapper;

pub type ThreadId = usize;

pub use self::{
    allocator::Allocator,
    binary_patches::install_binary_patches,
    context::ArmCoreContext,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    function::{EmulatedFunction, EmulatedFunctionParam, JumpTo, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId},
};
