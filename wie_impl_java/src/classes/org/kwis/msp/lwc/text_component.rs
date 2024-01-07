use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{JavaClassProto, JavaContextArg};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent {}

impl TextComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn set_max_length(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<TextComponent>, max_length: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::<init>({:?}, {})", &this, max_length);

        Ok(())
    }
}
