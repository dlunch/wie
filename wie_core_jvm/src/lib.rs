#![no_std]
extern crate alloc;

use alloc::{boxed::Box, format, rc::Rc, string::String, vec::Vec};
use core::{
    cell::{RefCell, RefMut},
    time::Duration,
};

use java_runtime::{get_class_proto, Runtime};
use java_runtime_base::{JavaResult, MethodBody};
use jvm::{Class, Jvm, JvmCallback, JvmResult};
use jvm_impl::{ClassImpl, JvmDetailImpl};

use wie_backend::SystemHandle;
use wie_impl_java::{get_class_proto as get_wie_class_proto, JavaContext};

pub type JvmCoreResult<T> = anyhow::Result<T>;

#[derive(Clone)]
struct JvmCoreRuntime;

#[async_trait::async_trait(?Send)]
impl Runtime for JvmCoreRuntime {
    async fn sleep(&self, _duration: Duration) {
        todo!()
    }
    async fn r#yield(&self) {
        todo!()
    }

    fn spawn(&self, _callback: Box<dyn JvmCallback>) {
        todo!()
    }

    fn now(&self) -> u64 {
        todo!()
    }

    fn encode_str(&self, _s: &str) -> Vec<u8> {
        todo!()
    }

    fn decode_str(&self, _bytes: &[u8]) -> String {
        todo!()
    }

    fn load_resource(&self, _name: &str) -> Option<Vec<u8>> {
        todo!()
    }

    fn println(&self, _s: &str) {
        todo!()
    }
}

#[derive(Clone)]
pub struct JvmCore {
    jvm: Rc<RefCell<Jvm>>,
}

impl JvmCore {
    pub fn new(system: &SystemHandle) -> Self {
        let jvm = Jvm::new(JvmDetailImpl::new(Self::get_class_loader(system)));

        Self {
            jvm: Rc::new(RefCell::new(jvm)),
        }
    }

    fn get_class_loader(system: &SystemHandle) -> impl Fn(&str) -> JvmResult<Option<Box<dyn Class>>> {
        let system_clone = system.clone();
        move |class_name| {
            tracing::debug!("Loading class {}", class_name);

            if let Some(x) = Self::load_class_from_impl(&system_clone, class_name)? {
                Ok(Some(x))
            } else {
                Self::load_class_from_resource(&system_clone, class_name)
            }
        }
    }

    fn load_class_from_impl(system: &SystemHandle, class_name: &str) -> JvmCoreResult<Option<Box<dyn Class>>> {
        if let Some(x) = get_class_proto(class_name) {
            let class = ClassImpl::from_class_proto(class_name, x, Box::new(JvmCoreRuntime) as Box<_>);

            Ok(Some(Box::new(class)))
        } else if let Some(x) = get_wie_class_proto(class_name) {
            let context = JvmCoreContext { system: system.clone() };

            let class = ClassImpl::from_class_proto(class_name, x, Box::new(context) as Box<_>);

            Ok(Some(Box::new(class)))
        } else {
            Ok(None)
        }
    }

    fn load_class_from_resource(system: &SystemHandle, class_name: &str) -> JvmCoreResult<Option<Box<dyn Class>>> {
        let path = format!("{}.class", class_name);
        let resource = system.resource();

        if let Some(x) = resource.id(&path) {
            let class_data = resource.data(x);

            Ok(Some(Box::new(ClassImpl::from_classfile(class_data)?)))
        } else {
            Ok(None)
        }
    }

    pub fn jvm(&mut self) -> RefMut<'_, Jvm> {
        self.jvm.borrow_mut()
    }
}

#[derive(Clone)]
struct JvmCoreContext {
    system: SystemHandle,
}

impl JavaContext for JvmCoreContext {
    fn system(&mut self) -> &mut SystemHandle {
        &mut self.system
    }

    fn spawn(&mut self, _callback: Box<dyn MethodBody<anyhow::Error, dyn JavaContext>>) -> JavaResult<()> {
        todo!()
    }
}
