#![no_std]
extern crate alloc;

mod filesystem;
mod jvm;
mod native_executor;
mod platform;

pub use self::{
    filesystem::MemoryFilesystem,
    jvm::{run_jvm_test, run_jvm_test_with_system},
    native_executor::TestNativeExecutor,
    platform::{TestClock, TestPlatform, TestPlatformEvent},
};
