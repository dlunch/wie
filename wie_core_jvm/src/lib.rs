#![no_std]
extern crate alloc;

use alloc::{fmt::Debug, format, rc::Rc};
use core::cell::RefCell;

use jvm::Jvm;

use wie_backend::{task, AsyncCallable, Backend};

#[derive(Clone, Default)]
pub struct JvmCore {
    jvm: Rc<RefCell<Jvm>>,
}

#[allow(dead_code)]
pub struct JvmClass<'a> {
    core: &'a mut JvmCore,
}

impl JvmCore {
    pub fn new() -> Self {
        Self {
            jvm: Rc::new(RefCell::new(Jvm::new())),
        }
    }

    pub fn spawn<C, R, E>(&mut self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static,
        R: 'static,
        E: Debug + 'static,
    {
        task::spawn(callable)
    }

    pub fn load_class(&mut self, backend: &Backend, class_name: &str) -> anyhow::Result<JvmClass<'_>> {
        let path = format!("{}.class", class_name.replace('.', "/"));

        let resource = backend.resource();
        let resource_id = resource.id(&path).unwrap();
        let class_data = resource.data(resource_id);

        self.jvm.borrow_mut().load_class(class_data)?;

        todo!()
    }
}
