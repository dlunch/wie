use alloc::{boxed::Box, vec};

use jvm::JavaValue;

use wie_backend::task;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JavaWord},
    method::MethodBody,
    proxy::{JavaObjectProxy, JvmClassInstanceProxy},
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
        tracing::debug!(
            "Thread::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&target.class_instance)
        );

        context.jvm().put_field(
            &this.class_instance,
            "target",
            "Ljava/lang/Runnable;",
            JavaValue::Object(Some(target.class_instance)),
        )?;

        Ok(())
    }

    async fn start(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("Thread::start({:#x})", context.instance_raw(&this.class_instance));

        struct ThreadStartProxy {
            runnable: JavaObjectProxy<Runnable>,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for ThreadStartProxy {
            #[tracing::instrument(name = "thread", fields(thread = self.runnable.ptr_instance), skip_all)]
            async fn call(&self, context: &mut dyn JavaContext, _: &[JavaWord]) -> Result<JavaWord, JavaError> {
                tracing::trace!("Thread start");

                context.call_method(&self.runnable.cast(), "run", "()V", &[]).await?;

                Ok(0)
            }
        }

        let target = context.jvm().get_field(&this.class_instance, "target", "Ljava/lang/Runnable;")?;
        let target = JavaObjectProxy::new(context.instance_raw(target.as_object_ref().unwrap()));

        context.spawn(Box::new(ThreadStartProxy { runnable: target }))?;

        Ok(())
    }

    async fn sleep(context: &mut dyn JavaContext, a0: i32, a1: i32) -> JavaResult<i32> {
        tracing::debug!("Thread::sleep({:#x}, {:#x})", a0, a1);
        context.sleep(a0 as u64).await;

        Ok(0)
    }

    async fn r#yield(_: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("Thread::yield()");
        task::yield_now().await;

        Ok(0)
    }

    async fn set_priority(_: &mut dyn JavaContext, this: JavaObjectProxy<Thread>, new_priority: i32) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Thread::setPriority({:#x}, {:#x})", this.ptr_instance, new_priority);

        Ok(())
    }
}
