mod window;

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    string::String,
    vec::Vec,
};

use crate::time::Time;

use self::window::Window;

pub struct Backend {
    resource: Rc<RefCell<Resource>>,
    window: Rc<RefCell<Window>>,
    time: Rc<RefCell<Time>>,
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        Self {
            resource: Rc::new(RefCell::new(Resource::new())),
            window: Rc::new(RefCell::new(Window::new())),
            time: Rc::new(RefCell::new(Time::new())),
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
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            window: self.window.clone(),
            time: self.time.clone(),
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
        self.files.push((path.to_string(), data));
    }

    pub fn id(&self, path: &str) -> Option<u32> {
        for (id, file) in self.files.iter().enumerate() {
            if file.0 == path {
                return Some(id as _);
            }
        }

        None
    }

    pub fn size(&self, id: u32) -> u32 {
        self.files[id as usize].1.len() as _
    }

    pub fn data(&self, id: u32) -> &[u8] {
        &self.files[id as usize].1
    }
}
