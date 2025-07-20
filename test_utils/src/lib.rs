#![no_std]
extern crate alloc;

mod jvm;
mod platform;

pub use self::{
    jvm::run_jvm_test,
    platform::{TestPlatform, TestPlatformEvent},
};
