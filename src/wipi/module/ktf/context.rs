use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::core::arm::allocator::Allocator;

pub struct ContextStorage {
    pub allocator: Allocator,
}

pub struct Context {
    storage: Rc<RefCell<ContextStorage>>,
}

impl Context {
    pub fn new(allocator: Allocator) -> Self {
        Self {
            storage: Rc::new(RefCell::new(ContextStorage { allocator })),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<ContextStorage> {
        self.storage.borrow_mut()
    }

    pub fn alloc(&self, size: u32) -> anyhow::Result<u32> {
        self.borrow_mut()
            .allocator
            .alloc(size)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}
