pub mod canvas;
mod window;

use std::{
    cell::{Ref, RefCell, RefMut},
    future::Future,
    rc::Rc,
    string::String,
    vec::Vec,
};

use wie_base::{Event, Module};

use crate::{executor::Executor, time::Time};

use self::{
    canvas::{Canvas, Image},
    window::Window,
};

pub struct Backend {
    resource: Rc<RefCell<Resource>>,
    time: Rc<RefCell<Time>>,
    screen_canvas: Rc<RefCell<Canvas>>,
    events: Rc<RefCell<Vec<Event>>>,
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        let screen_canvas = Canvas::from_image(Image::from_size(240, 320)); // TODO hardcoded size

        Self {
            resource: Rc::new(RefCell::new(Resource::new())),
            time: Rc::new(RefCell::new(Time::new())),
            screen_canvas: Rc::new(RefCell::new(screen_canvas)),
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        (*self.resource).borrow()
    }

    pub fn resource_mut(&self) -> RefMut<'_, Resource> {
        (*self.resource).borrow_mut()
    }

    pub fn time(&self) -> Ref<'_, Time> {
        (*self.time).borrow()
    }

    pub fn screen_canvas(&self) -> RefMut<'_, Canvas> {
        (*self.screen_canvas).borrow_mut()
    }

    pub fn events(&self) -> RefMut<'_, Vec<Event>> {
        (*self.events).borrow_mut()
    }

    pub fn run<M>(self, module: M) -> anyhow::Result<()>
    where
        M: Module + 'static,
    {
        let mut executor = Executor::new(module);

        Backend::run_task(&mut executor, &self.time(), |module| module.start())?;

        let screen_canvas = self.screen_canvas();
        let window = Window::new(screen_canvas.width(), screen_canvas.height());
        core::mem::drop(screen_canvas);

        window.run(
            || Ok::<_, anyhow::Error>(()),
            move |buffer| {
                executor.tick(&self.time())?;

                self.events().push(Event::Redraw);

                let canvas = self.screen_canvas();
                let rgb32 = canvas
                    .raw_rgba()
                    .chunks(4)
                    .map(|rgba8888| {
                        let rgba32 = u32::from_be_bytes(rgba8888.try_into().unwrap());
                        rgba32 >> 8
                    })
                    .collect::<Vec<_>>();
                buffer.copy_from_slice(&rgb32); // TODO is it better to introduce new vec?

                Ok(())
            },
        )
    }

    fn run_task<F, Fut>(executor: &mut Executor, time: &Time, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut dyn Module) -> Fut + 'static,
        Fut: Future<Output = anyhow::Result<()>> + 'static,
    {
        executor.run(time, || {
            let executor = Executor::current();
            let mut module = executor.module_mut();

            f(module.as_mut())
        })
    }
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            time: self.time.clone(),
            screen_canvas: self.screen_canvas.clone(),
            events: self.events.clone(),
        }
    }
}

pub struct Resource {
    files: Vec<(String, Vec<u8>)>,
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, path: &str, data: Vec<u8>) {
        tracing::debug!("Adding resource {}, {}b", path, data.len());

        self.files.push((path.to_string(), data));
    }

    pub fn id(&self, path: &str) -> Option<u32> {
        tracing::trace!("Looking for resource {}", path);

        for (id, file) in self.files.iter().enumerate() {
            if file.0 == path {
                return Some(id as _);
            }
        }

        tracing::warn!("No such resource {}", path);

        None
    }

    pub fn size(&self, id: u32) -> u32 {
        self.files[id as usize].1.len() as _
    }

    pub fn data(&self, id: u32) -> &[u8] {
        &self.files[id as usize].1
    }
}
