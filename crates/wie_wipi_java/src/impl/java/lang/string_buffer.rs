use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodAccessFlag, JavaMethodProto},
    JavaContext, JavaObjectProxy, JavaResult,
};

// class java.lang.StringBuffer
pub struct StringBuffer {}

impl StringBuffer {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
                    Self::append,
                    JavaMethodAccessFlag::NONE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, _instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub java.lang.StringBuffer::<init>()");

        Ok(())
    }

    async fn append(_: &mut dyn JavaContext, _instance: JavaObjectProxy, _a1: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub java.lang.StringBuffer::append()");

        Ok(JavaObjectProxy::new(0))
    }
}
