use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(_: &mut dyn JavaContext, _a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub HandsetProperty::getSystemProperty({:#x})", _a0.ptr_instance);

        Ok(JavaObjectProxy::new(0))
    }
}
