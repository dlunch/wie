use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use super::runtime::JavaBridgeContext;

pub struct ContextStorage {
    pub java_bridge_context: JavaBridgeContext,
}

pub struct Context {
    storage: Rc<RefCell<ContextStorage>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(ContextStorage {
                java_bridge_context: JavaBridgeContext::new(),
            })),
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
