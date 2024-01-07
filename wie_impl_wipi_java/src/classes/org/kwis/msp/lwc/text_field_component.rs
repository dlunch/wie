use alloc::vec;

use java_runtime::classes::java::lang::String;
use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.lwc.TextFieldComponent
pub struct TextFieldComponent {}

impl TextFieldComponent {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(
        _: &mut Jvm,
        _: &mut WIPIJavaContxt,
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
