use alloc::boxed::Box;

use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Result as JvmResult};

use wie_backend::System;

// TODO can we merge this and wipi's one?
pub trait SKVMJavaContextBase: DynClone + Send + Sync {
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, SKVMJavaContext>>) -> JvmResult<()>;
}

clone_trait_object!(SKVMJavaContextBase);

pub(crate) type SKVMJavaClassProto = JavaClassProto<dyn SKVMJavaContextBase>;
pub(crate) type SKVMJavaContext = dyn SKVMJavaContextBase;

#[cfg(test)]
pub mod test {
    use alloc::boxed::Box;

    use java_class_proto::MethodBody;
    use jvm::{JavaError, Result as JvmResult};

    use wie_backend::System;

    use crate::context::SKVMJavaContextBase;

    #[derive(Clone)]
    pub struct DummyContext;

    impl SKVMJavaContextBase for DummyContext {
        fn system(&mut self) -> &mut System {
            todo!()
        }

        fn spawn(&mut self, _callback: Box<dyn MethodBody<JavaError, dyn SKVMJavaContextBase>>) -> JvmResult<()> {
            todo!()
        }
    }
}
