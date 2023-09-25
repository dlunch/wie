use alloc::{format, vec};

use wie_backend::canvas::{PixelType, Rgb8Pixel};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    method::TypeConverter,
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::{Display, Font, Image},
    JavaFieldAccessFlag,
};

bitflags::bitflags! {
    struct Anchor: u32 {
        const TOP = 0;
        const HCENTER = 1;
        const VCENTER = 2;
        const LEFT = 4;
        const RIGHT = 8;
        const BOTTOM = 32;
        const BASELINE = 64;
    }
}

impl TypeConverter<Anchor> for Anchor {
    fn to_rust(_: &mut dyn JavaContext, raw: u32) -> Anchor {
        Anchor::from_bits_retain(raw)
    }

    fn from_rust(_: &mut dyn JavaContext, rust: Anchor) -> u32 {
        rust.bits()
    }
}

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
                JavaFieldProto::new("rgb", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, display: JavaObjectProxy<Display>) -> JavaResult<()> {
        let log = format!(
            "org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x})",
            this.ptr_instance, display.ptr_instance
        );
        tracing::debug!("{}", log); // splitted format as tracing macro doesn't like variable named `display` https://github.com/tokio-rs/tracing/issues/2332

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
        tracing::debug!(
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
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:#x})", this.ptr_instance);

        let font = context.instantiate("Lorg/kwis/msp/lcdui/Font;").await?.cast();
        context.call_method(&font.cast(), "<init>", "()V", &[]).await?;

        Ok(font)
    }

    async fn set_color(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, rgb: u32) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:#x}, {})", this.ptr_instance, rgb);

        context.put_field(&this.cast(), "rgb", rgb)?;

        Ok(())
    }

    async fn set_alpha(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a1: u32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setAlpha({:#x}, {})", this.ptr_instance, a1);

        Ok(())
    }

    async fn set_clip(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a0: u32, a1: u32, a2: u32, a3: u32) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::setClip({:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            a0,
            a1,
            a2,
            a3
        );

        Ok(())
    }

    async fn fill_rect(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, x: u32, y: u32, width: u32, height: u32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::fillRect({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            x,
            y,
            width,
            height
        );

        let rgb = context.get_field(&this.cast(), "rgb")?;

        let image = Self::image(context, &this).await?;
        let mut canvas = Image::canvas(context, &image)?;

        canvas.draw_rect(x, y, width, height, Rgb8Pixel::to_color(rgb));

        Ok(())
    }

    async fn draw_image(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Graphics>,
        img: JavaObjectProxy<Image>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            this.ptr_instance,
            img.ptr_instance,
            x,
            y,
            anchor.0
        );

        let src_canvas = Image::image(context, &img)?;

        let image = Self::image(context, &this).await?;
        let mut canvas = Image::canvas(context, &image)?;

        let x_delta = if anchor.contains(Anchor::HCENTER) {
            -((src_canvas.width() / 2) as i32)
        } else if anchor.contains(Anchor::RIGHT) {
            -(src_canvas.width() as i32)
        } else {
            0
        };

        let y_delta = if anchor.contains(Anchor::VCENTER) {
            -((src_canvas.height() / 2) as i32)
        } else if anchor.contains(Anchor::BOTTOM) {
            -(src_canvas.height() as i32)
        } else {
            0
        };

        let x = (x + x_delta).max(0);
        let y = (y + y_delta).max(0);

        canvas.draw(x as u32, y as u32, src_canvas.width(), src_canvas.height(), &*src_canvas, 0, 0);

        Ok(())
    }

    async fn get_clip_x(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_y(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_width(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipWidth({:#x})", this.ptr_instance);

        let w = context.get_field(&this.cast(), "w")?;

        Ok(w)
    }

    async fn get_clip_height(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<u32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipHeight({:#x})", this.ptr_instance);

        let h = context.get_field(&this.cast(), "h")?;

        Ok(h)
    }

    async fn image(context: &mut dyn JavaContext, this: &JavaObjectProxy<Graphics>) -> JavaResult<JavaObjectProxy<Image>> {
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
