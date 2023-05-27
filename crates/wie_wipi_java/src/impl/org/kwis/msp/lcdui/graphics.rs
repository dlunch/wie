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
                JavaMethodProto::new("setColor", "(I)V", Self::set_color),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Graphics::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::debug!("Graphics::getFont({:#x})", a0.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;")?;
        context.call_method(&font, "<init>", "()V", &[]).await?;

        Ok(font)
    }

    async fn set_color(_: &mut dyn JavaContext, a0: JavaObjectProxy, a1: u32) -> JavaResult<()> {
        log::debug!("Graphics::setColor({:#x}, {})", a0.ptr_instance, a1);

        Ok(())
    }

    async fn fill_rect(_: &mut dyn JavaContext, a0: JavaObjectProxy, a1: u32, a2: u32, a3: u32, a4: u32) -> JavaResult<()> {
        log::debug!("Graphics::fillRect({:#x}, {}, {}, {}, {})", a0.ptr_instance, a1, a2, a3, a4);

        Ok(())
    }
}
