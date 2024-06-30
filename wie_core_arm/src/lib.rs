#![no_std]
extern crate alloc;

mod allocator;
mod context;
mod core;
mod engine;
mod error;
mod function;
mod future;

pub type ArmCoreResult<T> = Result<T, error::ArmCoreError>;

pub use self::{
    allocator::Allocator,
    core::ArmCore,
    error::ArmCoreError,
    function::{EmulatedFunction, EmulatedFunctionParam},
};
