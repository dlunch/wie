#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod function;

pub use self::{
    allocator::Allocator,
    core::{ArmCore, RUN_FUNCTION_LR},
    function::{EmulatedFunction, EmulatedFunctionParam, ResultWriter},
};
