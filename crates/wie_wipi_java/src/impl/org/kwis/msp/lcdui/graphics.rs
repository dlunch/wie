use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Graphics
pub struct Graphics {}

impl Graphics {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getFont", "()Lorg/kwis/msp/lcdui/Font;", Self::get_font),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Graphics::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::debug!("Graphics::get_font({:#x})", a0.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;")?;
        context.call_method(&font, "<init>", "()V", &[]).await?;

        Ok(font)
    }
}
