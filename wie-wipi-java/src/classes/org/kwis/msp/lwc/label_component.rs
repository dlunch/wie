use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msp::lcdui::Image;

// class org.kwis.msp.lwc.LabelComponent
#[java_class(name = "org/kwis/msp/lwc/LabelComponent", extends = "org/kwis/msp/lwc/Component", public)]
pub struct LabelComponent;

#[java_methods]
impl LabelComponent {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    #[method(constructor, public)]
    async fn init_with_label(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    #[method(constructor, public)]
    async fn init_with_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?}, {image:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }

    #[method(constructor, public)]
    async fn init_with_image_resource(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        resource: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?}, {resource:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }
}
