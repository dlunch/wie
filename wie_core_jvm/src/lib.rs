#![no_std]
extern crate alloc;

use alloc::{boxed::Box, fmt::Debug, format, rc::Rc};
use core::cell::RefCell;

use jvm::{Class, Jvm, JvmResult};
use jvm_impl::{ClassImpl, JvmDetailImpl};

use wie_backend::{task, AsyncCallable, Backend};

pub type JvmCoreResult<T> = anyhow::Result<T>;

#[derive(Clone)]
pub struct JvmCore {
    jvm: Rc<RefCell<Jvm>>,
}

impl JvmCore {
    pub fn new(backend: &Backend) -> Self {
        let jvm = Jvm::new(JvmDetailImpl::new(Self::get_class_loader(backend)));

        Self {
            jvm: Rc::new(RefCell::new(jvm)),
        }
    }

    fn get_class_loader(backend: &Backend) -> impl Fn(&str) -> JvmResult<Option<Box<dyn Class>>> {
        let backend = backend.clone();
        move |class_name| {
            let path = format!("{}.class", class_name.replace('.', "/"));

            let resource = backend.resource();

            if let Some(x) = resource.id(&path) {
                let class_data = resource.data(x);

                Ok(Some(Box::new(ClassImpl::from_classfile(class_data)?)))
            } else {
                Ok(None)
            }
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

    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn invoke_static(&mut self, main_class_name: &str, method_name: &str, descriptor: &str) -> JvmCoreResult<()> {
        let mut jvm = self.jvm.borrow_mut();

        jvm.invoke_static(main_class_name, method_name, descriptor, []).await?;

        Ok(())
    }
}
