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
            methods: vec![
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, JavaMethodFlag::NONE),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn key_notify(_: &mut dyn JavaContext, this: JavaObjectProxy<Component>, r#type: i32, chr: i32) -> JavaResult<i32> {
        tracing::warn!(
            "stub org.kwis.msp.lwc.Component::keyNotify({:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            r#type,
            chr
        );

        Ok(1)
    }

    async fn set_focus(_: &mut dyn JavaContext, this: JavaObjectProxy<Component>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({:#x})", this.ptr_instance,);

        Ok(())
    }

    async fn get_height(_: &mut dyn JavaContext, this: JavaObjectProxy<Component>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({:#x})", this.ptr_instance,);

        Ok(0)
    }
}
