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

pub enum Event {
    Update,
    Redraw,
    Keyboard(u32),
}
