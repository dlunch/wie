#![no_std]

extern crate alloc;

pub mod methods;
pub mod util;

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum WIPIKey {
    UP = -1,
    DOWN = -6,
    LEFT = -2,
    RIGHT = -5,
    FIRE = -8, // Ok

    NUM0 = 48,
    NUM1 = 49,
    NUM2 = 50,
    NUM3 = 51,
    NUM4 = 52,
    NUM5 = 53,
    NUM6 = 54,
    NUM7 = 55,
    NUM8 = 56,
    NUM9 = 57,
    HASH = 35, // #
    STAR = 42, // *
}

#[derive(Debug)]
pub enum Event {
    Redraw,
    Keydown(WIPIKey),
    Keyup(WIPIKey),
}
