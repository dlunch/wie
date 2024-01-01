mod audio;
pub mod database;
mod resource;
pub mod screen;
pub mod time;

use alloc::{collections::VecDeque, rc::Rc};
use core::cell::{Ref, RefCell, RefMut};

use wie_base::Event;

use crate::{extract_zip, platform::Platform};

use self::{audio::Audio, database::DatabaseRepository, resource::Resource, time::Time};

pub struct System {
    platform: Box<dyn Platform>,
    database: DatabaseRepository,
    resource: Resource,
    events: VecDeque<Event>,
    audio: Audio,
}

impl System {
    pub fn new(app_id: &str, platform: Box<dyn Platform>) -> Self {
        Self {
            platform,
            database: DatabaseRepository::new(app_id),
            resource: Resource::new(),
            events: VecDeque::new(),
            audio: Audio::new(),
        }
    }
}

#[derive(Clone)]
pub struct SystemHandle {
    system: Rc<RefCell<System>>,
}

impl SystemHandle {
    pub fn new(system: System) -> Self {
        Self {
            system: Rc::new(RefCell::new(system)),
        }
    }

    pub fn database(&self) -> RefMut<'_, DatabaseRepository> {
        RefMut::map(self.system.borrow_mut(), |s| &mut s.database)
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        Ref::map(self.system.borrow(), |s| &s.resource)
    }

    pub fn time(&self) -> Time {
        Time::new(self.clone())
    }

    pub fn platform(&self) -> RefMut<'_, Box<dyn Platform>> {
        RefMut::map(self.system.borrow_mut(), |s| &mut s.platform)
    }

    pub fn audio(&self) -> RefMut<'_, Audio> {
        RefMut::map(self.system.borrow_mut(), |s| &mut s.audio)
    }

    pub fn pop_event(&self) -> Option<Event> {
        self.system.borrow_mut().events.pop_front()
    }

    pub fn push_event(&self, event: Event) {
        self.system.borrow_mut().events.push_back(event);
    }

    pub fn mount_zip(&self, zip: &[u8]) -> anyhow::Result<()> {
        let files = extract_zip(zip)?;

        let resource = &mut self.system.borrow_mut().resource;
        for (path, data) in files {
            resource.add(&path, data);
        }

        Ok(())
    }
}
