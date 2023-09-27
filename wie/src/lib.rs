#![no_std]
extern crate alloc;

use alloc::boxed::Box;

use wie_backend::{
    canvas::{ArgbPixel, ImageBuffer},
    App, Archive, Backend, Executor, WindowProxy,
};

pub struct Wie {
    app: Box<dyn App>,
    backend: Backend,
    executor: Executor,
}

impl Wie {
    pub fn new(archive: Box<dyn Archive>, window_proxy: WindowProxy) -> anyhow::Result<Self> {
        let canvas = ImageBuffer::<ArgbPixel>::new(window_proxy.width(), window_proxy.height());

        let mut backend = Backend::new(Box::new(canvas), window_proxy);

        let mut app = archive.load_app(&mut backend)?;

        let executor = Executor::new();

        app.start()?;

        Ok(Self { app, backend, executor })
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        self.executor
            .tick(&self.backend.time())
            .map_err(|x| anyhow::anyhow!("{}\n{}", x, self.app.crash_dump()))
    }

    pub fn send_event(&mut self, event: wie_base::Event) {
        self.backend.push_event(event)
    }
}
