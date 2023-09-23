#![no_std]

extern crate alloc;

pub mod methods;
pub mod util;

pub trait Module {
    fn start(&mut self);
}

pub enum Event {
    Redraw,
    Update,
}
