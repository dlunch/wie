#![no_std]
extern crate alloc;

mod jvm;
mod platform;
mod runtime;

pub use self::{jvm::test_jvm, platform::TestPlatform};
