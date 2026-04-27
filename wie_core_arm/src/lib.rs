#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod function;
mod native_hooks;
pub mod stdlib;
mod thread;
mod thread_wrapper;

#[cfg(not(target_arch = "wasm32"))]
mod gdb;

pub type ThreadId = usize;

pub use self::{
    allocator::Allocator,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    function::{EmulatedFunction, EmulatedFunctionParam, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId},
    native_hooks::install_native_hooks,
};
