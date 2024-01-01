#![no_std]
extern crate alloc;

mod base;
pub mod r#impl;
mod method;

pub use self::base::{WIPICContext, WIPICError, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};
