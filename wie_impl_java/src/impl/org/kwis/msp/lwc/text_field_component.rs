use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
    r#impl::java::lang::String,
};

// class org.kwis.msp.lwc.TextFieldComponent
pub struct TextFieldComponent {}

impl TextFieldComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(
        _: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<TextFieldComponent>,
        data: JvmClassInstanceHandle<String>,
        constraint: i32,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lwc.TextFieldComponent::<init>({:?}, {:?}, {:?})",
            &this,
            &data,
            constraint
        );

        Ok(())
    }
}
