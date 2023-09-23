#![no_std]

extern crate alloc;

pub mod methods;
pub mod util;

pub trait Module {
    fn start(&mut self);
    fn crash_dump(&self) -> alloc::string::String;
}

pub enum Event {
    Redraw,
    Update,
}
