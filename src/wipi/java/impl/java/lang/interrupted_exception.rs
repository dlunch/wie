use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaObjectProxy, JavaResult};

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("InterruptedException::<init>{:#x}", instance.ptr_instance);

        Ok(())
    }
}
