#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod function;
mod thread;
mod thread_wrapper;

pub use self::{
    allocator::Allocator,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult},
    function::{EmulatedFunction, EmulatedFunctionParam, ResultWriter},
};
