use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaObjectProxy, JavaResult};

// class java.lang.Thread
pub struct Thread {}

impl Thread {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("<init>", "(Ljava/lang/Runnable;)V", Self::init_1),
                JavaMethodProto::new("start", "()V", Self::start),
            ],
            fields: vec![],
        }
    }

    fn init(_: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    fn init_1(_: &mut JavaContext, instance: JavaObjectProxy, a0: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x}, {:#x})", instance.ptr_instance, a0.ptr_instance);

        Ok(())
    }

    fn start(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Thread::start");

        Ok(())
    }
}
