#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod app;
mod loader;
mod runtime;

pub use loader::{is_ktf_archive_loaded, load_ktf_archive};
