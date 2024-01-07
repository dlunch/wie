#![no_std]
extern crate alloc;

pub mod api;
mod base;
mod method;

pub use self::base::{WIPICContext, WIPICError, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};
