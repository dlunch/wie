use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextFieldComponent
pub struct TextFieldComponent;

impl TextFieldComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextFieldComponent",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "(Ljava/lang/String;I)V",
                Self::init,
                MethodAccessFlags::PUBLIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextFieldComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextFieldComponent::<init>({this:?}, {data:?}, {constraint:?})");

        jvm.invoke_special(
            &this,
            "org/kwis/msp/lwc/TextComponent",
            "<init>",
            "(Ljava/lang/String;I)V",
            (data, constraint),
        )
        .await
    }
}
