use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::TextComponent};

// class org.kwis.msp.lwc.TextBoxComponent
#[java_class(name = "org/kwis/msp/lwc/TextBoxComponent", extends = "org/kwis/msp/lwc/TextComponent", public)]
pub struct TextBoxComponent {
    #[field(private)]
    initialHeight: i32,
}

#[java_methods]
impl TextBoxComponent {
    #[method(constructor, public)]
    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {constraint:?})");

        jvm.invoke_special(
            &this,
            "org/kwis/msp/lwc/TextComponent",
            "<init>",
            "(Ljava/lang/String;I)V",
            (data, constraint),
        )
        .await
    }

    #[method(constructor, public)]
    async fn init_with_height(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {constraint}, {height})");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/TextBoxComponent",
                "<init>",
                "(Ljava/lang/String;I)V",
                (data, constraint),
            )
            .await?;

        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextBoxComponent", "initialHeight", "I", height)
            .await
    }

    #[method(protected)]
    async fn calc_preferred_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::calcPreferredSize({this:?}, {width})");
        let (natural_width, height) = TextComponent::measure_text(jvm, &this, Some(width)).await?;
        let min_height: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/TextBoxComponent", "initialHeight", "I").await?;
        let preferred_width = if width >= 0 { width } else { natural_width };
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefW", "I", preferred_width)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefH", "I", height.max(min_height))
            .await
    }

    #[method(public)]
    async fn paint_content(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::paintContent({this:?}, {g:?})");
        TextComponent::draw_text(jvm, this, g, true).await
    }
}
