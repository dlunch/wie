#![no_std]
extern crate alloc;

mod base;
mod graphics;
mod kernel;

pub use self::{
    base::{CContextBase, CContextMethod, CMethodBody, CResult, CContext},
    graphics::get_graphics_method_table,
    kernel::get_kernel_method_table,
};
