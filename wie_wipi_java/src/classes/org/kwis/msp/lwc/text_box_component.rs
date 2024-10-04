use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextBoxComponent
pub struct TextBoxComponent;

impl TextBoxComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextBoxComponent",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default())],
            fields: vec![],
        }
    }

    async fn init(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lwc.TextBoxComponent::<init>({:?}, {:?}, {:?})",
            &this,
            &data,
            constraint
        );

        Ok(())
    }
}
