#![no_std]
extern crate alloc;

mod jvm;
mod platform;

pub use self::{jvm::test_jvm, platform::TestPlatform};
