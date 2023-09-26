#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod app;
mod runtime;

pub use self::app::KtfWipiApp;
