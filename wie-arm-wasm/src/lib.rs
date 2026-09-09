#![no_std]
extern crate alloc;

mod codegen;

#[cfg(all(target_arch = "wasm32", feature = "compiler-runtime"))]
pub mod compiler;
#[cfg(all(target_arch = "wasm32", not(feature = "compiler-runtime")))]
mod runtime;

pub use codegen::{WasmArtifact, compile};
#[cfg(all(target_arch = "wasm32", not(feature = "compiler-runtime")))]
pub use runtime::WasmExecutor;

#[cfg(test)]
extern crate std;
