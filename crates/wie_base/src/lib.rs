#![no_std]

extern crate alloc;

pub mod methods;
pub mod util;

use alloc::boxed::Box;

#[async_trait::async_trait(?Send)]
pub trait Module {
    async fn start(&mut self) -> anyhow::Result<()>;
    fn crash_dump(&self) -> alloc::string::String;
}

#[derive(Debug)]
pub enum Event {
    Update,
    Redraw,
    Keydown(u32),
    Keyup(u32),
}
