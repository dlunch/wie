#![no_std]
extern crate alloc;

mod filesystem;
mod jvm;
#[cfg(not(target_arch = "wasm32"))]
mod native_executor;
mod platform;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native_executor::TestNativeExecutor;
pub use self::{
    filesystem::MemoryFilesystem,
    jvm::{run_jvm_test, run_jvm_test_with_system},
    platform::{TestClock, TestPlatform, TestPlatformEvent},
};
