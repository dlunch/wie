#![no_std]

extern crate alloc;

pub mod methods;
pub mod util;

#[derive(Debug)]
pub enum Event {
    Redraw,
    Keydown(u32),
    Keyup(u32),
}
