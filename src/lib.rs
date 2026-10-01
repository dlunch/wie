extern crate alloc;

mod loader;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
mod audio_sink;
#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod database;
#[cfg(not(target_arch = "wasm32"))]
mod filesystem;
#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
mod window;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub use cli::run;
#[cfg(not(target_arch = "wasm32"))]
pub use database::DatabaseRepository;
#[cfg(not(target_arch = "wasm32"))]
pub use filesystem::DiskFilesystem;
pub use loader::{AppMetadata, extract_app_metadata, load_emulator};
