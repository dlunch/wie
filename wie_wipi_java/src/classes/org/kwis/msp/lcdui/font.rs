use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto, JavaResult};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.lcdui.Font
pub struct Font {}

impl Font {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Lorg/kwis/msp/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getFont", "(III)Lorg/kwis/msp/lcdui/Font;", Self::get_font, MethodAccessFlags::STATIC),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn cl_init(jvm: &mut Jvm, _: &mut WIPIJavaContext) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<clinit>");

        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_SMALL", "I", 8).await?;

        Ok(())
    }

    async fn init(_: &mut Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Font>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::<init>({:?})", &this);

        Ok(())
    }

    async fn get_height(_: &mut Jvm, _: &mut WIPIJavaContext) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getHeight");

        Ok(12) // TODO: hardcoded
    }

    async fn get_default_font(jvm: &mut Jvm, _: &mut WIPIJavaContext) -> JavaResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &mut Jvm, _: &mut WIPIJavaContext, face: i32, style: i32, size: i32) -> JavaResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getFont({:?}, {:?}, {:?})", face, style, size);

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }
}
