mod audio;
pub mod canvas;
pub mod database;
mod resource;
pub mod screen;
pub mod time;

use alloc::{collections::VecDeque, rc::Rc};
use core::cell::{Ref, RefCell, RefMut};

use wie_base::Event;

use crate::{extract_zip, platform::Platform};

use self::{
    audio::Audio,
    canvas::{ArgbPixel, Canvas, ImageBuffer},
    database::DatabaseRepository,
    resource::Resource,
    screen::Screen,
    time::Time,
};

#[derive(Clone)]
pub struct System {
    platform: Rc<Box<dyn Platform>>,
    database: Rc<RefCell<DatabaseRepository>>,
    resource: Rc<RefCell<Resource>>,
    screen_canvas: Rc<RefCell<Box<dyn Canvas>>>,
    events: Rc<RefCell<VecDeque<Event>>>,
    audio: Rc<RefCell<Audio>>,
}

impl System {
    pub fn new(app_id: &str, platform: Box<dyn Platform>) -> Self {
        let screen = platform.screen();
        let screen_canvas = ImageBuffer::<ArgbPixel>::new(screen.width(), screen.height());

        Self {
            platform: Rc::new(platform),
            database: Rc::new(RefCell::new(DatabaseRepository::new(app_id))),
            resource: Rc::new(RefCell::new(Resource::new())),
            screen_canvas: Rc::new(RefCell::new(Box::new(screen_canvas))),
            events: Rc::new(RefCell::new(VecDeque::new())),
            audio: Rc::new(RefCell::new(Audio::new())),
        }
    }

    pub fn database(&self) -> RefMut<'_, DatabaseRepository> {
        (*self.database).borrow_mut()
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        (*self.resource).borrow()
    }

    pub fn time(&self) -> Time<'_> {
        Time::new(&**self.platform)
    }

    pub fn screen_canvas(&self) -> RefMut<'_, Box<dyn Canvas>> {
        (*self.screen_canvas).borrow_mut()
    }

    pub fn screen(&self) -> &dyn Screen {
        self.platform.screen()
    }

    pub fn audio(&self) -> RefMut<'_, Audio> {
        (*self.audio).borrow_mut()
    }

    pub fn pop_event(&self) -> Option<Event> {
        (*self.events).borrow_mut().pop_front()
    }

    pub fn push_event(&self, event: Event) {
        (*self.events).borrow_mut().push_back(event);
    }

    pub fn repaint(&self) -> anyhow::Result<()> {
        self.screen().repaint(&**self.screen_canvas())
    }

    pub fn mount_zip(&self, zip: &[u8]) -> anyhow::Result<()> {
        let files = extract_zip(zip)?;

        let mut resource = (*self.resource).borrow_mut();
        for (path, data) in files {
            resource.add(&path, data);
        }

        Ok(())
    }
}
