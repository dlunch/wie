use std::{cell::RefCell, rc::Rc};

use crate::core::arm::allocator::Allocator;

pub struct ContextStorage {
    pub allocator: Allocator,
}

pub type Context = Rc<RefCell<ContextStorage>>;
