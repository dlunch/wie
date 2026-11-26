use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.lwc.TextFieldComponent
#[java_class(name = "org/kwis/msp/lwc/TextFieldComponent", extends = "org/kwis/msp/lwc/TextComponent", public)]
pub struct TextFieldComponent;

#[java_methods]
impl TextFieldComponent {
    #[method(constructor, public)]
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
