use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, Default::default()),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.TextComponent::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_max_length(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>, max_length: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::<init>({:?}, {})", &this, max_length);

        Ok(())
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lwc.TextFieldComponent::<init>({:?})", &this);

        let result = JavaLangString::from_rust_string(jvm, "temp").await?;

        Ok(result.into())
    }
}
