use wie_backend::{task::SleepFuture, Backend};
use wie_impl_java::{get_class_proto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodBody, JavaMethodProto, JavaResult};

use jvm::{Class, JavaValue, Jvm, JvmResult};
use jvm_impl::{ClassImpl, FieldImpl, JvmDetailImpl, MethodBody, MethodImpl, RustMethodBody};

pub struct TestContext {
    jvm: Jvm,
}

impl TestContext {
    pub fn new() -> Self {
        let jvm = Jvm::new(JvmDetailImpl::new(Self::load_class));

        Self { jvm }
    }

    fn load_class(class_name: &str) -> JvmResult<Option<Box<dyn Class>>> {
        let class_proto = get_class_proto(class_name);
        if let Some(x) = class_proto {
            let class = ClassImpl::new(class_name, Self::load_methods(x.methods), Self::load_fields(x.fields));

            Ok(Some(Box::new(class)))
        } else {
            Ok(None)
        }
    }

    fn load_methods(methods: Vec<JavaMethodProto>) -> Vec<MethodImpl> {
        methods
            .into_iter()
            .map(|x| MethodImpl::new(&x.name, &x.descriptor, Self::load_method_body(x.body)))
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

    fn load_method_body(body: JavaMethodBody) -> MethodBody {
        struct MethodProxy {
            body: JavaMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl RustMethodBody<anyhow::Error, JavaValue> for MethodProxy {
            async fn call(&self, jvm: &mut Jvm, args: &[JavaValue]) -> Result<JavaValue, anyhow::Error> {
                struct InnerContext<'a> {
                    jvm: &'a mut Jvm,
                }

                impl<'a> JavaContext for InnerContext<'a> {
                    fn jvm(&mut self) -> &mut Jvm {
                        &mut self.jvm
                    }

                    fn backend(&mut self) -> &mut Backend {
                        todo!()
                    }

                    fn spawn(&mut self, _callback: JavaMethodBody) -> JavaResult<()> {
                        todo!()
                    }

                    fn sleep(&mut self, _duration: u64) -> SleepFuture {
                        todo!()
                    }
                }

                let args = args.iter().cloned().collect();

                self.body.call(&mut InnerContext { jvm }, args).await
            }
        }

        MethodBody::Rust(Box::new(MethodProxy { body }))
    }
}

impl JavaContext for TestContext {
    fn jvm(&mut self) -> &mut Jvm {
        &mut self.jvm
    }

    fn backend(&mut self) -> &mut Backend {
        todo!()
    }

    fn spawn(&mut self, _callback: JavaMethodBody) -> JavaResult<()> {
        todo!()
    }

    fn sleep(&mut self, _duration: u64) -> SleepFuture {
        todo!()
    }
}
