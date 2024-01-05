use alloc::{boxed::Box, format, string::String, vec};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
    method::MethodBody,
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
                JavaMethodProto::new("<init>", "(Ljava/lang/Runnable;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("start", "()V", Self::start, JavaMethodFlag::NONE),
                JavaMethodProto::new("sleep", "(J)V", Self::sleep, JavaMethodFlag::NATIVE),
                JavaMethodProto::new("yield", "()V", Self::r#yield, JavaMethodFlag::NATIVE),
                JavaMethodProto::new("setPriority", "(I)V", Self::set_priority, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("target", "Ljava/lang/Runnable;", crate::JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, mut this: JvmClassInstanceHandle<Self>, target: JvmClassInstanceHandle<Runnable>) -> JavaResult<()> {
        tracing::debug!("Thread::<init>({:?}, {:?})", &this, &target);

        context.jvm().put_field(&mut this, "target", "Ljava/lang/Runnable;", target)?;

        Ok(())
    }

    async fn start(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("Thread::start({:?})", &this);

        struct ThreadStartProxy {
            thread_id: String,
            runnable: JvmClassInstanceHandle<Runnable>,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for ThreadStartProxy {
            #[tracing::instrument(name = "thread", fields(thread = self.thread_id), skip_all)]
            async fn call(&self, context: &mut dyn JavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                tracing::trace!("Thread start");

                context
                    .jvm()
                    .invoke_virtual(&self.runnable, "java/lang/Runnable", "run", "()V", [])
                    .await?;

                Ok(JavaValue::Void)
            }
        }

        let runnable = context.jvm().get_field(&this, "target", "Ljava/lang/Runnable;")?;

        context.spawn(Box::new(ThreadStartProxy {
            thread_id: format!("{:?}", &runnable),
            runnable,
        }))?;

        Ok(())
    }

    async fn sleep(context: &mut dyn JavaContext, duration: i64) -> JavaResult<i32> {
        tracing::debug!("Thread::sleep({:?})", duration);

        let until = context.system().platform().now() + duration as _;
        context.system().sleep(until).await;

        Ok(0)
    }

    async fn r#yield(context: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("Thread::yield()");
        context.system().yield_now().await;

        Ok(0)
    }

    async fn set_priority(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Thread>, new_priority: i32) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Thread::setPriority({:?}, {:?})", &this, new_priority);

        Ok(())
    }
}
