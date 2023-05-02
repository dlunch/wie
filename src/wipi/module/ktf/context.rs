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
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}
