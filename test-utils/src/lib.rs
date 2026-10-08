#![no_std]
extern crate alloc;

mod filesystem;
mod jvm;
#[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
mod native_executor;
mod platform;

#[cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]
pub use self::native_executor::TestNativeExecutor;
pub use self::{
    filesystem::MemoryFilesystem,
    jvm::{run_jvm_test, run_jvm_test_with_system},
    platform::{TestClock, TestPlatform, TestPlatformEvent},
};
