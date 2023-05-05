use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::core::arm::allocator::Allocator;

use super::runtime::JavaBridgeContext;

pub struct ContextStorage {
    pub allocator: Allocator,
    pub java_bridge_context: JavaBridgeContext,
}

pub struct Context {
    storage: Rc<RefCell<ContextStorage>>,
}

impl Context {
    pub fn new(allocator: Allocator) -> Self {
        Self {
            storage: Rc::new(RefCell::new(ContextStorage {
                allocator,
                java_bridge_context: JavaBridgeContext::new(),
            })),
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
