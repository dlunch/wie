#![no_std]
extern crate alloc;

use alloc::boxed::Box;

use anyhow::Context;

use wie_backend::{Backend, Executor, Window};
use wie_base::{App, Event};
use wie_vendor_ktf::{is_ktf_archive_loaded, load_ktf_archive};

fn load_archive(file: &[u8], backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
    backend.add_resources_from_zip(file)?;

    if is_ktf_archive_loaded(backend) {
        Ok(Box::new(load_ktf_archive(backend)?))
    } else {
        anyhow::bail!("Unknown vendor")
    }
}

pub fn start(file: &[u8]) -> anyhow::Result<()> {
    let mut backend = Backend::new();

    let mut app = load_archive(file, &mut backend)?;

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
