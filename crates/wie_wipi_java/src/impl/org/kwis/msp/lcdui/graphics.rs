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
                JavaMethodProto::new("drawImage", "(Lorg/kwis/msp/lcdui/Image;III)V", Self::draw_image),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Graphics::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Graphics::getFont({:#x})", a0.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;")?;
        context.call_method(&font, "<init>", "()V", &[]).await?;

        Ok(font)
    }

    async fn set_color(_: &mut dyn JavaContext, a0: JavaObjectProxy, a1: u32) -> JavaResult<()> {
        log::warn!("stub Graphics::setColor({:#x}, {})", a0.ptr_instance, a1);

        Ok(())
    }

    async fn fill_rect(_: &mut dyn JavaContext, a0: JavaObjectProxy, a1: u32, a2: u32, a3: u32, a4: u32) -> JavaResult<()> {
        log::warn!("stub Graphics::fillRect({:#x}, {:#x}, {}, {}, {})", a0.ptr_instance, a1, a2, a3, a4);

        Ok(())
    }

    async fn draw_image(_: &mut dyn JavaContext, a0: JavaObjectProxy, a1: JavaObjectProxy, a2: u32, a3: u32, a4: u32) -> JavaResult<()> {
        log::warn!(
            "stub Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            a0.ptr_instance,
            a1.ptr_instance,
            a2,
            a3,
            a4
        );

        Ok(())
    }

    async fn get_clip_x(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub Graphics::getClipX({:#x})", a0.ptr_instance);

        Ok(0)
    }

    async fn get_clip_y(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub Graphics::getClipY({:#x})", a0.ptr_instance);

        Ok(0)
    }

    async fn get_clip_width(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub Graphics::getClipWidth({:#x})", a0.ptr_instance);

        Ok(0)
    }

    async fn get_clip_height(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub Graphics::getClipHeight({:#x})", a0.ptr_instance);

        Ok(0)
    }
}
