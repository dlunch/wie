#![no_std]
extern crate alloc;

pub mod api;
mod context;
mod method;

pub use self::context::{WIPICContext, WIPICError, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};
