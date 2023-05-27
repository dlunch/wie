#![no_std]
extern crate alloc;

mod base;
mod database;
mod graphics;
mod kernel;
mod media;
mod method;

pub use self::{
    base::{CContext, CMethodBody, CResult},
    database::get_database_method_table,
    graphics::get_graphics_method_table,
    kernel::get_kernel_method_table,
    media::get_media_method_table,
};
