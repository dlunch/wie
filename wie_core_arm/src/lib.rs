#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod function;
mod thread;
mod thread_wrapper;

#[cfg(not(target_arch = "wasm32"))]
mod gdb;

pub type ThreadId = usize;

pub use self::{
    allocator::Allocator,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    engine::ArmRegister,
    function::{EmulatedFunction, EmulatedFunctionParam, ResultWriter},
};
