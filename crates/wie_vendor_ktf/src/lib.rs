#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod module;
mod runtime;

pub use self::module::KtfWipiModule;
