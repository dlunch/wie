#![no_std]
extern crate alloc;

use alloc::boxed::Box;

use anyhow::Context;

use wie_backend::{Archive, Backend, Executor, Window};
use wie_base::Event;

pub fn start(archive: Box<dyn Archive>) -> anyhow::Result<()> {
    let mut backend = Backend::new();

    let mut app = archive.load_app(&mut backend)?;

    let mut executor = Executor::new();

    app.start()?;

    let event_loop = backend.window().event_loop();
    Window::run(event_loop, move |event| {
        match event {
            Event::Update => executor.tick(&backend.time()).with_context(|| app.crash_dump())?,
            _ => backend.push_event(event),
        }

        anyhow::Ok(())
    });
}
