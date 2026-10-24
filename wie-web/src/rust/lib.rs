#![no_std]
extern crate alloc;
extern crate std;

#[cfg(target_arch = "wasm32")]
mod aot;
#[cfg(target_arch = "wasm32")]
mod audio_sink;
#[cfg(target_arch = "wasm32")]
mod database;
#[cfg(target_arch = "wasm32")]
mod filesystem;
mod host;
#[cfg(target_arch = "wasm32")]
mod indexed_db_store;
#[cfg(target_arch = "wasm32")]
mod runtime;
#[cfg(target_arch = "wasm32")]
mod save_migration;
mod util;
#[cfg(target_arch = "wasm32")]
mod window;
