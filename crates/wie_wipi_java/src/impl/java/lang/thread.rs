use alloc::vec;

use wie_backend::task;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodProto, JavaResult},
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

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn init_1(context: &mut dyn JavaContext, instance: JavaObjectProxy, runnable: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::<init>({:#x}, {:#x})", instance.ptr_instance, runnable.ptr_instance);

        context.put_field(&instance, "runnable", runnable.ptr_instance)?;

        Ok(())
    }

    async fn start(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Thread::start");

        let _runnable = JavaObjectProxy::new(context.get_field(&instance, "runnable")?);

        task::spawn(async {
            log::debug!("Thread::run");

            todo!()

            // context.call_method(&runnable, "run", "()V", &[]).await?; // TODO
        });

        Ok(())
    }

    async fn sleep(_: &mut dyn JavaContext, a0: u32, a1: u32) -> JavaResult<u32> {
        log::debug!("Thread::sleep({:#x}, {:#x})", a0, a1);
        task::sleep(a1 as u64).await;

        Ok(0)
    }

    async fn r#yield(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::debug!("Thread::yield()");
        task::yield_now().await;

        Ok(0)
    }
}
