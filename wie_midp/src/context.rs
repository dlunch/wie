use alloc::boxed::Box;

use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, MethodBody};
use jvm::{JavaError, Result as JvmResult};

use wie_backend::System;

// TODO can we merge this and wipi's one?
pub trait MIDPJavaContextBase: DynClone + Send + Sync {
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: Box<dyn MethodBody<JavaError, MIDPJavaContext>>) -> JvmResult<()>;
}

clone_trait_object!(MIDPJavaContextBase);

pub(crate) type MIDPJavaClassProto = JavaClassProto<dyn MIDPJavaContextBase>;
pub(crate) type MIDPJavaContext = dyn MIDPJavaContextBase;

#[cfg(test)]
pub mod test {
    use alloc::boxed::Box;

    use java_class_proto::MethodBody;
    use jvm::{JavaError, Result as JvmResult};

    use wie_backend::System;

    use crate::context::MIDPJavaContextBase;

    #[derive(Clone)]
    pub struct DummyContext;

    impl MIDPJavaContextBase for DummyContext {
        fn system(&mut self) -> &mut System {
            todo!()
        }

        fn spawn(&mut self, _callback: Box<dyn MethodBody<JavaError, dyn MIDPJavaContextBase>>) -> JvmResult<()> {
            todo!()
        }
    }
}
