use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use core::cell::{Ref, RefCell, RefMut};

struct Storage {
    resource: Resource,
    scheduler: Scheduler,
}

pub struct Backend {
    storage: Rc<RefCell<Storage>>,
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(Storage {
                resource: Resource::new(),
                scheduler: Scheduler::new(),
            })),
        }
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        Ref::map(self.storage.borrow(), |x| &x.resource)
    }

    pub fn scheduler(&mut self) -> RefMut<'_, Scheduler> {
        RefMut::map(self.storage.borrow_mut(), |x| &mut x.scheduler)
    }
}

impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
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

pub trait Task {
    fn run_some(&mut self) -> anyhow::Result<()>;
}

pub struct Scheduler {
    tasks: Vec<Box<dyn Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule<T>(&mut self, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.push(Box::new(task))
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while !self.tasks.is_empty() {
            for task in &mut self.tasks {
                task.run_some()?;
            }
        }

        Ok(())
    }
}
