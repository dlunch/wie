use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lwc.Component
pub struct Component {}

impl Component {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn key_notify(_: &mut dyn JavaContext, this: JavaObjectProxy<Component>, r#type: i32, chr: i32) -> JavaResult<i32> {
        tracing::warn!(
            "stub org.kwis.msp.lwc.Component::key_notify({:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            r#type,
            chr
        );

        Ok(1)
    }
}
