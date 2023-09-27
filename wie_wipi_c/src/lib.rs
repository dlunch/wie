#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod base;
pub mod r#impl;
mod method;

pub use self::base::{CContext, CError, CMemoryId, CMethodBody, CResult};
