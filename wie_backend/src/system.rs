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

use self::{audio::Audio, database::DatabaseRepository, resource::Resource, time::Time};

#[derive(Clone)]
pub struct System {
    platform: Rc<RefCell<Box<dyn Platform>>>,
    database: Rc<RefCell<DatabaseRepository>>,
    resource: Rc<RefCell<Resource>>,
    events: Rc<RefCell<VecDeque<Event>>>,
    audio: Rc<RefCell<Audio>>,
}

impl System {
    pub fn new(app_id: &str, platform: Box<dyn Platform>) -> Self {
        Self {
            platform: Rc::new(RefCell::new(platform)),
            database: Rc::new(RefCell::new(DatabaseRepository::new(app_id))),
            resource: Rc::new(RefCell::new(Resource::new())),
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

    pub fn time(&self) -> Time {
        Time::new(self.platform.clone())
    }

    pub fn platform(&self) -> RefMut<'_, Box<dyn Platform>> {
        (*self.platform).borrow_mut()
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

    pub fn mount_zip(&self, zip: &[u8]) -> anyhow::Result<()> {
        let files = extract_zip(zip)?;

        let mut resource = (*self.resource).borrow_mut();
        for (path, data) in files {
            resource.add(&path, data);
        }

        Ok(())
    }
}
