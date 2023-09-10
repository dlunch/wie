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
            fields: vec![
                JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("h", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, display: JavaObjectProxy<Display>) -> JavaResult<()> {
        log::debug!(
            "org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x})",
            this.ptr_instance,
            display.ptr_instance
        );

        let width = context.get_field(&display.cast(), "m_w")?;
        let height = context.get_field(&display.cast(), "m_h")?;

        context.put_field(&this.cast(), "w", width)?;
        context.put_field(&this.cast(), "h", height)?;

        Ok(())
    }

    async fn init_with_image(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Graphics>,
        image: JavaObjectProxy<Image>,
        a0: u32,
        a1: u32,
        width: u32,
        height: u32,
    ) -> JavaResult<()> {
        log::debug!(
            "org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            image.ptr_instance,
            a0,
            a1,
            width,
            height
        );

        context.put_field(&this.cast(), "img", image.ptr_instance)?;
        context.put_field(&this.cast(), "w", width)?;
        context.put_field(&this.cast(), "h", height)?;

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
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Graphics>,
        img: JavaObjectProxy<Image>,
        x: u32,
        y: u32,
        anchor: u32,
    ) -> JavaResult<()> {
        log::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            img.ptr_instance,
            x,
            y,
            anchor
        );

        let src_buf = Image::get_buf(context, &img)?;
        let src_width = context.get_field(&img.cast(), "w")?;
        let src_height = context.get_field(&img.cast(), "h")?;

        let image = Self::get_image(context, &this).await?;
        let mut canvas = Image::get_canvas(context, &image)?;

        canvas.draw(x, y, src_width, src_height, &src_buf, 0, 0, src_width);

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

    async fn get_image(context: &mut dyn JavaContext, this: &JavaObjectProxy<Graphics>) -> JavaResult<JavaObjectProxy<Image>> {
        let image = JavaObjectProxy::new(context.get_field(&this.cast(), "img")?);

        if image.ptr_instance != 0 {
            Ok(image)
        } else {
            let width = context.get_field(&this.cast(), "w")?;
            let height = context.get_field(&this.cast(), "h")?;

            let image = JavaObjectProxy::new(
                context
                    .call_static_method(
                        "org/kwis/msp/lcdui/Image",
                        "createImage",
                        "(II)Lorg/kwis/msp/lcdui/Image;",
                        &[width, height],
                    )
                    .await?,
            );

            context.put_field(&this.cast(), "img", image.ptr_instance)?;

            Ok(image)
        }
    }
}
