use alloc::{boxed::Box, format, string::String, vec};

use jvm::{ClassInstanceRef, JavaValue};

use wie_backend::task;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JvmClassInstanceProxy,
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

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, target: JvmClassInstanceProxy<Runnable>) -> JavaResult<()> {
        tracing::debug!("Thread::<init>({:?}, {:?})", &this, &target);

        context.jvm().put_field(&this, "target", "Ljava/lang/Runnable;", target)?;

        Ok(())
    }

    async fn start(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("Thread::start({:?})", &this);

        struct ThreadStartProxy {
            thread_id: String,
            runnable: JvmClassInstanceProxy<Runnable>,
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

        let target: ClassInstanceRef = context.jvm().get_field(&this, "target", "Ljava/lang/Runnable;")?;
        let runnable = target.into();

        context.spawn(Box::new(ThreadStartProxy {
            thread_id: format!("{:?}", &runnable),
            runnable,
        }))?;

        Ok(())
    }

    async fn sleep(context: &mut dyn JavaContext, a0: i64) -> JavaResult<i32> {
        tracing::debug!("Thread::sleep({:?})", a0);
        context.sleep(a0 as u64).await;

        Ok(0)
    }

    async fn r#yield(_: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("Thread::yield()");
        task::yield_now().await;

        Ok(0)
    }

    async fn set_priority(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Thread>, new_priority: i32) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Thread::setPriority({:?}, {:?})", &this, new_priority);

        Ok(())
    }
}
