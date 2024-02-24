use alloc::boxed::Box;

use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Result as JvmResult};

use wie_backend::System;

pub trait WIPIJavaContextBase: DynClone {
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, WIPIJavaContext>>) -> JvmResult<()>;
}

clone_trait_object!(WIPIJavaContextBase);

pub(crate) type WIPIJavaClassProto = JavaClassProto<dyn WIPIJavaContextBase>;
pub(crate) type WIPIJavaContext = dyn WIPIJavaContextBase;

#[cfg(test)]
pub mod test {
    use alloc::boxed::Box;

    use java_class_proto::MethodBody;
    use jvm::{JavaError, Result as JvmResult};

    use wie_backend::System;

    use crate::context::WIPIJavaContextBase;

    #[derive(Clone)]
    pub struct DummyContext;

    impl WIPIJavaContextBase for DummyContext {
        fn system(&mut self) -> &mut System {
            todo!()
        }

        fn spawn(&mut self, _callback: Box<dyn MethodBody<JavaError, dyn WIPIJavaContextBase>>) -> JvmResult<()> {
            todo!()
        }
    }
}
