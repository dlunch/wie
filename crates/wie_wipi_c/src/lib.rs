#![no_std]
extern crate alloc;

mod base;
mod graphics;
mod kernel;

pub use self::{
    base::{CContext, CContextBase, CContextMethod, CMethodBody, CResult},
    graphics::get_graphics_method_table,
    kernel::get_kernel_method_table,
};
