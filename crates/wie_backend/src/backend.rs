mod scheduler;
mod window;

use alloc::{rc::Rc, string::String, vec::Vec};
use core::cell::{Ref, RefCell, RefMut};

use wie_base::Core;

pub use self::scheduler::{Task, TaskStatus};
use self::{scheduler::Scheduler, window::Window};

pub struct Backend {
    resource: Rc<RefCell<Resource>>,
    scheduler: Rc<RefCell<Scheduler>>,
    window: Rc<RefCell<Window>>,
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
            scheduler: Rc::new(RefCell::new(Scheduler::new())),
            window: Rc::new(RefCell::new(Window::new())),
        }
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        (*self.resource).borrow()
    }

    pub fn scheduler(&self) -> RefMut<'_, Scheduler> {
        self.scheduler.borrow_mut()
    }

    pub fn run(self, core: &mut dyn Core) -> anyhow::Result<()> {
        Scheduler::run(self, core)
    }
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            scheduler: self.scheduler.clone(),
            window: self.window.clone(),
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
