#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod function;
mod future;

pub use self::{
    allocator::Allocator,
    context::ArmCoreContext,
    core::{ArmCore, ArmCoreError, PEB_BASE},
    function::{EmulatedFunction, EmulatedFunctionParam},
};
