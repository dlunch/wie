use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::{Display, Font, Image},
    JavaFieldAccessFlag,
};

// class org.kwis.msp.lcdui.Graphics
pub struct Graphics {}

impl Graphics {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Image;IIII)V", Self::init_with_image, JavaMethodFlag::NONE),
                JavaMethodProto::new("getFont", "()Lorg/kwis/msp/lcdui/Font;", Self::get_font, JavaMethodFlag::NONE),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, JavaMethodFlag::NONE),
                JavaMethodProto::new("setAlpha", "(I)V", Self::set_alpha, JavaMethodFlag::NONE),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, JavaMethodFlag::NONE),
                JavaMethodProto::new("drawImage", "(Lorg/kwis/msp/lcdui/Image;III)V", Self::draw_image, JavaMethodFlag::NONE),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, display: JavaObjectProxy<Display>) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x})",
            this.ptr_instance,
            display.ptr_instance
        );

        Ok(())
    }

    async fn init_with_image(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<Graphics>,
        image: JavaObjectProxy<Image>,
        a0: u32,
        a1: u32,
        a2: u32,
        a3: u32,
    ) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            image.ptr_instance,
            a0,
            a1,
            a2,
            a3
        );

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<JavaObjectProxy<Font>> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:#x})", this.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;")?.cast();
        context.call_method(&font.cast(), "<init>", "()V", &[]).await?;

        Ok(font)
    }

    async fn set_color(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a1: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::setColor({:#x}, {})", this.ptr_instance, a1);

        Ok(())
    }

    async fn set_alpha(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a1: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::setAlpha({:#x}, {})", this.ptr_instance, a1);

        Ok(())
    }

    async fn set_clip(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a0: u32, a1: u32, a2: u32, a3: u32) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::setClip({:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            a0,
            a1,
            a2,
            a3
        );

        Ok(())
    }

    async fn fill_rect(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, x: u32, y: u32, width: u32, height: u32) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::fillRect({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn draw_image(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<Graphics>,
        img: JavaObjectProxy<Image>,
        x: u32,
        y: u32,
        anchor: u32,
    ) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            img.ptr_instance,
            x,
            y,
            anchor
        );

        Ok(())
    }

    async fn get_clip_x(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_y(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_width(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipWidth({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_height(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Graphics::getClipHeight({:#x})", this.ptr_instance);

        Ok(0)
    }
}
