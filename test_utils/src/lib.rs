#![no_std]
extern crate alloc;

mod filesystem;
mod jvm;
mod platform;

pub use self::{
    filesystem::MemoryFilesystem,
    jvm::run_jvm_test,
    platform::{TestPlatform, TestPlatformEvent},
};
