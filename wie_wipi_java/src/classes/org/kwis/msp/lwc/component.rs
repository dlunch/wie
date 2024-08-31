use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.Component
pub struct Component {}

impl Component {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/Component",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, Default::default()),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({:?}, {:?}, {:?})", &this, r#type, chr);

        Ok(true)
    }

    async fn set_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({:?})", &this,);

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({:?})", &this,);

        Ok(0)
    }
}
