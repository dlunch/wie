use alloc::{boxed::Box, vec};

use wie_backend::task;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JavaObjectProxy,
    JavaError,
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
        log::trace!("Thread::start({:#x})", instance.ptr_instance);

        let runnable = JavaObjectProxy::new(context.get_field(&instance, "runnable")?);

        context.spawn(Box::new(ThreadStartProxy { runnable }))?;

        Ok(())
    }

    async fn sleep(context: &mut dyn JavaContext, a0: u32, a1: u32) -> JavaResult<u32> {
        log::trace!("Thread::sleep({:#x}, {:#x})", a0, a1);
        context.sleep(a1 as u64).await;

        Ok(0)
    }

    async fn r#yield(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::trace!("Thread::yield()");
        task::yield_now().await;

        Ok(0)
    }
}

struct ThreadStartProxy {
    runnable: JavaObjectProxy,
}

#[async_trait::async_trait(?Send)]
impl MethodBody<JavaError> for ThreadStartProxy {
    async fn call(&self, context: &mut dyn JavaContext, _: &[u32]) -> Result<u32, JavaError> {
        log::debug!("Thread::run");

        context.call_method(&self.runnable, "run", "()V", &[]).await?;

        Ok(0)
    }
}
