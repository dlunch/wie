#![no_std]
extern crate alloc;

mod arm_core;
mod jvm_core;
mod platform;

pub use self::{arm_core::test_arm_core, jvm_core::test_jvm_core, platform::TestPlatform};
