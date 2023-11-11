use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Font
pub struct Font {}

impl Font {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Lorg/kwis/msp/lcdui/Font;",
                    Self::get_default_font,
                    JavaMethodFlag::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_SYSTEM", "I", crate::JavaFieldAccessFlag::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", crate::JavaFieldAccessFlag::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", crate::JavaFieldAccessFlag::STATIC),
            ],
        }
    }

    async fn cl_init(context: &mut dyn JavaContext, this: JavaObjectProxy<Font>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::<clinit>({:#x})", this.ptr_instance);

        context.put_static_field("org/kwis/msp/lcdui/Font", "FACE_SYSTEM", 0)?;
        context.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_PLAIN", 0)?;
        context.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_SMALL", 8)?;

        Ok(())
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Font>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn get_height(_: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::get_height");

        Ok(12) // TODO: hardcoded
    }

    async fn get_default_font(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::get_default_font");

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Font;").await?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance.cast())
    }
}
