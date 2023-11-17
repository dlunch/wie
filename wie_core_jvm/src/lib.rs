#![no_std]
extern crate alloc;

use alloc::{fmt::Debug, format, rc::Rc};
use core::cell::RefCell;

use jvm::{Class, ClassLoader, Jvm};

use wie_backend::{task, AsyncCallable, Backend};

pub type JvmCoreResult<T> = anyhow::Result<T>;

#[derive(Clone, Default)]
pub struct JvmCore {
    jvm: Rc<RefCell<Jvm>>,
}

impl JvmCore {
    pub fn new(backend: &Backend) -> Self {
        let mut jvm = Jvm::new();
        jvm.add_class_loader(JvmCoreClassLoader { backend: backend.clone() });

        Self {
            jvm: Rc::new(RefCell::new(jvm)),
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

    pub fn invoke_static_method(&mut self, main_class_name: &str, method_name: &str, descriptor: &str) -> JvmCoreResult<()> {
        let mut jvm = self.jvm.borrow_mut();

        jvm.invoke_static_method(main_class_name, method_name, descriptor)
    }
}

struct JvmCoreClassLoader {
    backend: Backend,
}

impl ClassLoader for JvmCoreClassLoader {
    fn load(&mut self, class_name: &str) -> jvm::JvmResult<Option<jvm::Class>> {
        let path = format!("{}.class", class_name.replace('.', "/"));

        let resource = self.backend.resource();
        let resource_id = resource.id(&path).unwrap();
        let class_data = resource.data(resource_id);

        Some(Class::from_classfile(class_data)).transpose()
    }
}
