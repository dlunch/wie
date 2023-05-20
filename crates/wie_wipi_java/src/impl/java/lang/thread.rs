use alloc::vec;

use wie_base::method::MethodImpl;

use crate::{
    base::{JavaClassProto, JavaContext, JavaError, JavaFieldProto, JavaMethodProto, JavaResult},
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
                JavaMethodProto::new("sleep", "(J)V", Self::sleep),
                JavaMethodProto::new("yield", "()V", Self::r#yield),
            ],
            fields: vec![JavaFieldProto::new("runnable", "Ljava/lang/Runnable;")],
        }
    }

    fn init(_: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    fn init_1(context: &mut JavaContext, instance: JavaObjectProxy, runnable: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x}, {:#x})", instance.ptr_instance, runnable.ptr_instance);

        context.put_field(&instance, "runnable", runnable.ptr_instance)?;

        Ok(())
    }

    fn start(context: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::start");

        let runnable = JavaObjectProxy::new(context.get_field(&instance, "runnable")?);

        context.task_schedule(
            (move |context: &mut JavaContext| {
                log::debug!("Thread::run");

                context.call_method(&runnable, "run", "()V", &[])?;

                Ok::<_, JavaError>(())
            })
            .into_body(),
        )?;

        Ok(())
    }

    fn sleep(_: &mut JavaContext, a0: u32, a1: u32) -> JavaResult<u32> {
        log::debug!("Thread::sleep({:#x}, {:#x})", a0, a1);

        Ok(0)
    }

    fn r#yield(_: &mut JavaContext) -> JavaResult<u32> {
        log::debug!("Thread::yield()");

        Ok(0)
    }
}
