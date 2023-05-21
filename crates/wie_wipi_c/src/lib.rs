#![no_std]
extern crate alloc;

mod base;
mod graphics;
mod kernel;
mod method;

pub use self::{
    base::{CContext, CContextMethod, CMethodBody, CResult},
    graphics::get_graphics_method_table,
    kernel::get_kernel_method_table,
};
