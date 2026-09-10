#![cfg(target_arch = "wasm32")]
#![no_std]
extern crate alloc;

mod codegen;

mod compiler;
mod runtime;

pub use codegen::{WasmArtifact, compile};
pub use runtime::WasmExecutor;
