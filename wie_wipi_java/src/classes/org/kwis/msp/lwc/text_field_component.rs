use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextFieldComponent
pub struct TextFieldComponent;

impl TextFieldComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextFieldComponent",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new("insert", "([CIII)V", Self::insert, Default::default()),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextFieldComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::<init>({this:?}, {data:?}, {constraint:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn insert(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextFieldComponent>,
        data: ClassInstanceRef<Array<u16>>,
        offset: i32,
        length: i32,
        index: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::insert({this:?}, {data:?}, {offset}, {length}, {index})");

        Ok(())
    }

    async fn set_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextFieldComponent>,
        data: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (data,))
            .await
    }
}
