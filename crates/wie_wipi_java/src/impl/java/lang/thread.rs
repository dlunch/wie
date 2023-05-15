use alloc::vec;

use wie_base::method::MethodImpl;

use crate::{
    base::{JavaClassProto, JavaContext, JavaError, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

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

    fn start(context: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Thread::start");

        context.schedule_task(
            (|_: &mut JavaContext| {
                log::debug!("Thread::run");

                Ok::<_, JavaError>(())
            })
            .into_body(),
        )?;

        Ok(())
    }
}
