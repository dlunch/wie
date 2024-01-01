use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
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

    async fn key_notify(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, r#type: i32, chr: i32) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({:?}, {:?}, {:?})", &this, r#type, chr);

        Ok(true)
    }

    async fn set_focus(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({:?})", &this,);

        Ok(())
    }

    async fn get_height(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({:?})", &this,);

        Ok(0)
    }
}
