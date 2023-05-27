use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class java.lang.Runtime
pub struct Runtime {}

impl Runtime {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getRuntime", "()Ljava/lang/Runtime;", Self::get_runtime),
                JavaMethodProto::new("totalMemory", "()J", Self::total_memory),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Runtime::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_runtime(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Runtime::get_runtime");

        let instance = context.instantiate("Ljava/lang/Runtime;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn total_memory(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::warn!("stub Runtime::total_memory");

        Ok(0x100000) // TODO: hardcoded
    }
}
