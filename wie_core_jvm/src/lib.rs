#![no_std]
extern crate alloc;

use alloc::{boxed::Box, fmt::Debug, format, rc::Rc, vec::Vec};
use core::cell::{RefCell, RefMut};

use jvm::{Class, JavaValue, Jvm, JvmResult};
use jvm_impl::{ClassImpl, FieldImpl, JvmDetailImpl, MethodBody, MethodImpl, RustMethodBody};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend,
};
use wie_impl_java::{get_class_proto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodBody, JavaMethodProto, JavaResult};

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
            tracing::debug!("Loading class {}", class_name);

            if let Some(x) = Self::load_class_from_impl(&backend, class_name)? {
                Ok(Some(x))
            } else {
                Self::load_class_from_resource(&backend, class_name)
            }
        }
    }

    fn load_class_from_impl(backend: &Backend, class_name: &str) -> JvmCoreResult<Option<Box<dyn Class>>> {
        let class_proto = get_class_proto(class_name);
        if let Some(x) = class_proto {
            let class = ClassImpl::new(class_name, Self::load_methods(backend, x.methods), Self::load_fields(x.fields));

            Ok(Some(Box::new(class)))
        } else {
            Ok(None)
        }
    }

    fn load_methods(backend: &Backend, methods: Vec<JavaMethodProto>) -> Vec<MethodImpl> {
        methods
            .into_iter()
            .map(|x| MethodImpl::new(&x.name, &x.descriptor, Self::load_method_body(backend, x.body)))
            .collect()
    }

    fn load_fields(fields: Vec<JavaFieldProto>) -> Vec<FieldImpl> {
        fields
            .into_iter()
            .scan(0, |index, x| {
                let field = FieldImpl::new(&x.name, &x.descriptor, x.access_flag == JavaFieldAccessFlag::STATIC, *index);
                *index += 1;

                Some(field)
            })
            .collect()
    }

    fn load_class_from_resource(backend: &Backend, class_name: &str) -> JvmCoreResult<Option<Box<dyn Class>>> {
        let path = format!("{}.class", class_name.replace('/', "."));
        let resource = backend.resource();

        if let Some(x) = resource.id(&path) {
            let class_data = resource.data(x);

            Ok(Some(Box::new(ClassImpl::from_classfile(class_data)?)))
        } else {
            Ok(None)
        }
    }

    fn load_method_body(backend: &Backend, body: JavaMethodBody) -> MethodBody {
        struct MethodProxy {
            body: JavaMethodBody,
            backend: Backend,
        }

        #[async_trait::async_trait(?Send)]
        impl RustMethodBody<anyhow::Error, JavaValue> for MethodProxy {
            async fn call(&self, jvm: &mut Jvm, args: Box<[JavaValue]>) -> Result<JavaValue, anyhow::Error> {
                let mut context = JvmCoreContext {
                    backend: self.backend.clone(),
                    jvm,
                };

                self.body.call(&mut context, args).await
            }
        }

        MethodBody::Rust(Box::new(MethodProxy {
            body,
            backend: backend.clone(),
        }))
    }

    pub fn spawn<C, R, E>(&mut self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static,
        R: 'static,
        E: Debug + 'static,
    {
        task::spawn(callable)
    }

    pub fn jvm(&mut self) -> RefMut<'_, Jvm> {
        self.jvm.borrow_mut()
    }
}

struct JvmCoreContext<'a> {
    backend: Backend,
    jvm: &'a mut Jvm,
}

impl<'a> JavaContext for JvmCoreContext<'a> {
    fn jvm(&mut self) -> &mut Jvm {
        self.jvm
    }

    fn backend(&mut self) -> &mut Backend {
        &mut self.backend
    }

    fn spawn(&mut self, _callback: JavaMethodBody) -> JavaResult<()> {
        todo!()
    }

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.backend.time().now() + duration;

        task::sleep(until)
    }
}
