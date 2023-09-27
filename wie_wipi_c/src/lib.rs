#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod base;
mod r#impl;
mod method;

pub use self::{
    base::{CContext, CError, CMemoryId, CMethodBody, CResult},
    r#impl::{
        database::get_database_method_table, graphics::get_graphics_method_table, kernel::get_kernel_method_table, media::get_media_method_table,
        stub::get_stub_method_table,
    },
};
