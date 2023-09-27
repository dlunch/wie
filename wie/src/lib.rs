#![no_std]
extern crate alloc;

use alloc::boxed::Box;

use wie_backend::{
    canvas::{ArgbPixel, Image, ImageBuffer},
    Archive, Backend, Executor, Window, WindowCallbackEvent,
};

pub fn start(archive: Box<dyn Archive>) -> anyhow::Result<()> {
    let canvas = ImageBuffer::<ArgbPixel>::new(240, 320); // TODO hardcoded size
    let window = Window::new(canvas.width(), canvas.height());

    let mut backend = Backend::new(Box::new(canvas), window.proxy());

    let mut app = archive.load_app(&mut backend)?;

    let mut executor = Executor::new();

    app.start()?;

    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => executor
                .tick(&backend.time())
                .map_err(|x| anyhow::anyhow!("{}\n{}", x, app.crash_dump()))?,
            WindowCallbackEvent::Event(x) => backend.push_event(x),
        }

        anyhow::Ok(())
    });
}
