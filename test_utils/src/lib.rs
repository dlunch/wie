#![no_std]
extern crate alloc;

mod arm_core;
mod jvm;
mod platform;

pub use self::{arm_core::test_arm_core, jvm::test_jvm, platform::TestPlatform};
