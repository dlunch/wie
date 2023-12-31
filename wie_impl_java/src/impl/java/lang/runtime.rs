use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JvmClassInstanceProxy,
};

// class java.lang.Runtime
pub struct Runtime {}

impl Runtime {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getRuntime", "()Ljava/lang/Runtime;", Self::get_runtime, JavaMethodFlag::STATIC),
                JavaMethodProto::new("totalMemory", "()J", Self::total_memory, JavaMethodFlag::NONE),
                JavaMethodProto::new("freeMemory", "()J", Self::free_memory, JavaMethodFlag::NONE),
                JavaMethodProto::new("gc", "()V", Self::gc, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Runtime>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Runtime::<init>({:?})", &this);

        Ok(())
    }

    async fn get_runtime(context: &mut dyn JavaContext) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.Runtime::getRuntime");

        let instance = context.jvm().instantiate_class("java/lang/Runtime").await?;
        context.jvm().invoke_special(&instance, "java/lang/Runtime", "<init>", "()V", []).await?;

        Ok(instance.into())
    }

    async fn total_memory(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Runtime>) -> JavaResult<i64> {
        tracing::warn!("stub java.lang.Runtime::totalMemory({:?})", &this);

        Ok(0x100000) // TODO: hardcoded
    }

    async fn free_memory(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Runtime>) -> JavaResult<i64> {
        tracing::warn!("stub java.lang.Runtime::freeMemory({:?})", &this);

        Ok(0x100000) // TODO: hardcoded
    }

    async fn gc(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Runtime>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Runtime::gc({:?})", &this);

        Ok(())
    }
}
