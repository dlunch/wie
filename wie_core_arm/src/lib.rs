#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod function;
mod future;

pub use self::{
    allocator::Allocator,
    core::{ArmCore, PEB_BASE},
    engine::ArmEngineError,
    function::{EmulatedFunction, EmulatedFunctionParam},
};
