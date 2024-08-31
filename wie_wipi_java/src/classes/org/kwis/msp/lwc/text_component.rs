use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent {}

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, Default::default())],
            fields: vec![],
        }
    }

    async fn set_max_length(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>, max_length: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::<init>({:?}, {})", &this, max_length);

        Ok(())
    }
}
