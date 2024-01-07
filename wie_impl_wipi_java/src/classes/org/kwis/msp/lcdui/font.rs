use alloc::vec;

use java_runtime_base::{JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.lcdui.Font
pub struct Font {}

impl Font {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, JavaMethodFlag::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Lorg/kwis/msp/lcdui/Font;",
                    Self::get_default_font,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new("getFont", "(III)Lorg/kwis/msp/lcdui/Font;", Self::get_font, JavaMethodFlag::STATIC),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_SYSTEM", "I", JavaFieldAccessFlag::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", JavaFieldAccessFlag::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", JavaFieldAccessFlag::STATIC),
            ],
        }
    }

    async fn cl_init(jvm: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<clinit>");

        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_SMALL", "I", 8).await?;

        Ok(())
    }

    async fn init(_: &mut Jvm, _: &mut WIPIJavaContxt, this: JvmClassInstanceHandle<Font>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::<init>({:?})", &this);

        Ok(())
    }

    async fn get_height(_: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getHeight");

        Ok(12) // TODO: hardcoded
    }

    async fn get_default_font(jvm: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &mut Jvm, _: &mut WIPIJavaContxt, face: i32, style: i32, size: i32) -> JavaResult<JvmClassInstanceHandle<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getFont({:?}, {:?}, {:?})", face, style, size);

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }
}
