#![no_std]
extern crate alloc;

mod filesystem;
mod jvm;
mod platform;

pub use self::{
    filesystem::MemoryFilesystem,
    jvm::{run_jvm_test, run_jvm_test_with_system},
    platform::{TestClock, TestPlatform, TestPlatformEvent},
};
