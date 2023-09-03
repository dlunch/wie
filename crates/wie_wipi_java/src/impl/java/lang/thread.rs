use alloc::{boxed::Box, vec};

use wie_backend::task;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JavaObjectProxy,
    r#impl::java::lang::Runnable,
    JavaError,
};

// class java.lang.Thread
pub struct Thread {}

impl Thread {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/Runnable;)V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("start", "()V", Self::start, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("sleep", "(J)V", Self::sleep, JavaMethodAccessFlag::NATIVE),
                JavaMethodProto::new("yield", "()V", Self::r#yield, JavaMethodAccessFlag::NATIVE),
            ],
            fields: vec![JavaFieldProto::new("runnable", "Ljava/lang/Runnable;", crate::JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Thread>, runnable: JavaObjectProxy<Runnable>) -> JavaResult<()> {
        log::trace!("Thread::<init>({:#x}, {:#x})", this.ptr_instance, runnable.ptr_instance);

        context.put_field(&this.cast(), "runnable", runnable.ptr_instance)?;

        Ok(())
    }

    async fn start(context: &mut dyn JavaContext, this: JavaObjectProxy<Thread>) -> JavaResult<()> {
        log::trace!("Thread::start({:#x})", this.ptr_instance);

        let runnable = JavaObjectProxy::new(context.get_field(&this.cast(), "runnable")?);

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
    runnable: JavaObjectProxy<Runnable>,
}

#[async_trait::async_trait(?Send)]
impl MethodBody<JavaError> for ThreadStartProxy {
    async fn call(&self, context: &mut dyn JavaContext, _: &[u32]) -> Result<u32, JavaError> {
        log::trace!("Thread::run");

        context.call_method(&self.runnable.cast(), "run", "()V", &[]).await?;

        Ok(0)
    }
}
