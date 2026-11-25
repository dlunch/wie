use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::TextComponent};

// class org.kwis.msp.lwc.TextBoxComponent
pub struct TextBoxComponent;

impl TextBoxComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextBoxComponent",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("calcPreferredSize", "(I)V", Self::calc_preferred_size, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "paintContent",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_content,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("initialHeight", "I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

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

    async fn paint_content(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::paintContent({this:?}, {g:?})");
        TextComponent::draw_text(jvm, this, g, true).await
    }
}
