use alloc::vec;

use wie_backend::CanvasHandle;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Graphics
pub struct Graphics {}

impl Graphics {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getFont", "()Lorg/kwis/msp/lcdui/Font;", Self::get_font, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "drawImage",
                    "(Lorg/kwis/msp/lcdui/Image;III)V",
                    Self::draw_image,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy, canvas_handle: CanvasHandle) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::<init>({:#x}, {})", this.ptr_instance, canvas_handle);

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:#x})", this.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;")?;
        context.call_method(&font, "<init>", "()V", &[]).await?;

        Ok(font)
    }

    async fn set_color(_: &mut dyn JavaContext, this: JavaObjectProxy, a1: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::setColor({:#x}, {})", this.ptr_instance, a1);

        Ok(())
    }

    async fn fill_rect(_: &mut dyn JavaContext, this: JavaObjectProxy, a1: u32, a2: u32, a3: u32, a4: u32) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::fillRect({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            a1,
            a2,
            a3,
            a4
        );

        Ok(())
    }

    async fn draw_image(_: &mut dyn JavaContext, this: JavaObjectProxy, a1: JavaObjectProxy, a2: u32, a3: u32, a4: u32) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            a1.ptr_instance,
            a2,
            a3,
            a4
        );

        Ok(())
    }

    async fn get_clip_x(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_y(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_width(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipWidth({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_height(_: &mut dyn JavaContext, this: JavaObjectProxy) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipHeight({:#x})", this.ptr_instance);

        Ok(0)
    }
}
