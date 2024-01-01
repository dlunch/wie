mod audio;
mod resource;

use alloc::{collections::VecDeque, rc::Rc};
use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
};

use wie_base::Event;

use crate::{executor::Executor, extract_zip, platform::Platform, AsyncCallable};

use self::{audio::Audio, resource::Resource};

pub struct SystemInner {
    resource: Resource,
    events: VecDeque<Event>,
    audio: Audio,
}

pub struct System {
    executor: Executor,
    platform: Rc<RefCell<Box<dyn Platform>>>,
    inner: Rc<RefCell<SystemInner>>,
}

impl System {
    pub fn new(platform: Box<dyn Platform>) -> Self {
        let platform = Rc::new(RefCell::new(platform));

        Self {
            executor: Executor::new(),
            platform,
            inner: Rc::new(RefCell::new(SystemInner {
                resource: Resource::new(),
                events: VecDeque::new(),
                audio: Audio::new(),
            })),
        }
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        let platform = self.platform.clone();
        self.executor.tick(move || {
            let platform = platform.borrow();

            platform.now()
        })
    }

    pub fn handle(&self) -> SystemHandle {
        SystemHandle {
            platform: self.platform.clone(),
            system_inner: self.inner.clone(),
        }
    }
}

#[derive(Clone)]
pub struct SystemHandle {
    platform: Rc<RefCell<Box<dyn Platform>>>,
    system_inner: Rc<RefCell<SystemInner>>,
}

impl SystemHandle {
    pub fn spawn<C, R, E>(&self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static,
        E: Debug,
    {
        let mut executor = Executor::current();
        executor.spawn(callable);
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        Ref::map(self.system_inner.borrow(), |s| &s.resource)
    }

    pub fn platform(&self) -> RefMut<'_, Box<dyn Platform>> {
        self.platform.borrow_mut()
    }

    pub fn audio(&self) -> RefMut<'_, Audio> {
        RefMut::map(self.system_inner.borrow_mut(), |s| &mut s.audio)
    }

    pub fn pop_event(&self) -> Option<Event> {
        self.system_inner.borrow_mut().events.pop_front()
    }

    pub fn push_event(&self, event: Event) {
        self.system_inner.borrow_mut().events.push_back(event);
    }

    pub fn mount_zip(&self, zip: &[u8]) -> anyhow::Result<()> {
        let files = extract_zip(zip)?;

        let resource = &mut self.system_inner.borrow_mut().resource;
        for (path, data) in files {
            resource.add(&path, data);
        }

        Ok(())
    }
}
