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

    fn init(_: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Runtime::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    fn get_runtime(context: &mut JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Runtime::get_runtime");

        let instance = context.instantiate("Ljava/lang/Runtime;")?;
        // context.call_method(&instance, "<init>", "()V", &[]).await?; // TODO

        Ok(instance)
    }

    fn total_memory(_: &mut JavaContext) -> JavaResult<u32> {
        log::debug!("Runtime::total_memory");

        Ok(0x100000) // TODO: hardcoded
    }
}
