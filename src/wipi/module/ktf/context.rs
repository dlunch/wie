use std::{cell::RefCell, rc::Rc};

pub struct ContextStorage {}

pub struct Context {
    storage: Rc<RefCell<ContextStorage>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(ContextStorage {})),
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}
