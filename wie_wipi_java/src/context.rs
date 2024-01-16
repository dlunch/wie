use alloc::boxed::Box;

use dyn_clone::{clone_trait_object, DynClone};

use java_class_proto::{JavaClassProto, JavaResult, MethodBody};

use wie_backend::SystemHandle;

pub trait WIPIJavaContextBase: DynClone {
    fn system(&mut self) -> &mut SystemHandle;
    fn spawn(&mut self, callback: Box<dyn MethodBody<anyhow::Error, WIPIJavaContext>>) -> JavaResult<()>;
}

clone_trait_object!(WIPIJavaContextBase);

pub(crate) type WIPIJavaClassProto = JavaClassProto<dyn WIPIJavaContextBase>;
pub(crate) type WIPIJavaContext = dyn WIPIJavaContextBase;

#[cfg(test)]
pub mod test {
    use alloc::boxed::Box;

    use java_class_proto::{JavaResult, MethodBody};

    use wie_backend::SystemHandle;

    use crate::context::WIPIJavaContextBase;

    #[derive(Clone)]
    pub struct DummyContext;

    impl WIPIJavaContextBase for DummyContext {
        fn system(&mut self) -> &mut SystemHandle {
            todo!()
        }

        fn spawn(&mut self, _callback: Box<dyn MethodBody<anyhow::Error, dyn WIPIJavaContextBase>>) -> JavaResult<()> {
            todo!()
        }
    }
}
