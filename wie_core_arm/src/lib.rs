#![no_std]
extern crate alloc;

mod allocator;
mod binary_patches;
mod context;
mod core;
mod engine;
mod function;
pub mod stdlib;
mod thread;
mod thread_wrapper;

#[cfg(not(target_arch = "wasm32"))]
mod gdb;

pub type ThreadId = usize;

pub use self::{
    allocator::Allocator,
    binary_patches::install_binary_patches,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    function::{EmulatedFunction, EmulatedFunctionParam, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId},
};
