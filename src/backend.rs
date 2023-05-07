use alloc::rc::Rc;
use core::cell::{Ref, RefCell};

struct Storage {
    resource: Resource,
}

pub struct Backend {
    storage: Rc<RefCell<Storage>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(Storage { resource: Resource::new() })),
        }
    }

    pub fn resource(&self) -> Ref<'_, Resource> {
        Ref::map(self.storage.borrow(), |x| &x.resource)
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
