use alloc::{format, vec, vec::Vec};

use bytemuck::cast_vec;
use jvm::JavaValue;

use wie_backend::canvas::{ImageBuffer, PixelType, Rgb8Pixel};

use java_runtime::classes::java::lang::String;
use java_runtime_base::{
    Array, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle, TypeConverter,
};

use jvm::Jvm;

use crate::{
    classes::org::kwis::msp::lcdui::{Display, Font, Image},
    JavaClassProto, JavaContextArg,
};

bitflags::bitflags! {
    struct Anchor: i32 {
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
    fn to_rust(_: &mut Jvm, raw: JavaValue) -> Anchor {
        let raw: i32 = raw.into();
        Anchor::from_bits_retain(raw)
    }

    fn from_rust(_: &mut Jvm, rust: Anchor) -> JavaValue {
        rust.bits().into()
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
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_by_rgb, JavaMethodFlag::NONE),
                JavaMethodProto::new("setFont", "(Lorg/kwis/msp/lcdui/Font;)V", Self::set_font, JavaMethodFlag::NONE),
                JavaMethodProto::new("setAlpha", "(I)V", Self::set_alpha, JavaMethodFlag::NONE),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, JavaMethodFlag::NONE),
                JavaMethodProto::new("drawLine", "(IIII)V", Self::draw_line, JavaMethodFlag::NONE),
                JavaMethodProto::new("drawRect", "(IIII)V", Self::draw_rect, JavaMethodFlag::NONE),
                JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, JavaMethodFlag::NONE),
                JavaMethodProto::new("drawImage", "(Lorg/kwis/msp/lcdui/Image;III)V", Self::draw_image, JavaMethodFlag::NONE),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, JavaMethodFlag::NONE),
                JavaMethodProto::new("clipRect", "(IIII)V", Self::clip_rect, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height, JavaMethodFlag::NONE),
                JavaMethodProto::new("getTranslateX", "()I", Self::get_translate_x, JavaMethodFlag::NONE),
                JavaMethodProto::new("getTranslateY", "()I", Self::get_translate_y, JavaMethodFlag::NONE),
                JavaMethodProto::new("translate", "(II)V", Self::translate, JavaMethodFlag::NONE),
                JavaMethodProto::new("setRGBPixels", "(IIII[III)V", Self::set_rgb_pixels, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("h", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("rgb", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        display: JvmClassInstanceHandle<Display>,
    ) -> JavaResult<()> {
        let log = format!("org.kwis.msp.lcdui.Graphics::<init>({:?}, {:?})", &this, &display);
        tracing::debug!("{}", log); // splitted format as tracing macro doesn't like variable named `display` https://github.com/tokio-rs/tracing/issues/2332

        let width: i32 = jvm.get_field(&display, "m_w", "I")?;
        let height: i32 = jvm.get_field(&display, "m_h", "I")?;

        jvm.put_field(&mut this, "w", "I", width)?;
        jvm.put_field(&mut this, "h", "I", height)?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_image(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        image: JvmClassInstanceHandle<Image>,
        a0: i32,
        a1: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::<init>({:?}, {:?}, {}, {}, {}, {})",
            &this,
            &image,
            a0,
            a1,
            width,
            height
        );

        jvm.put_field(&mut this, "img", "Lorg/kwis/msp/lcdui/Image;", image)?;
        jvm.put_field(&mut this, "w", "I", width)?;
        jvm.put_field(&mut this, "h", "I", height)?;

        Ok(())
    }

    async fn get_font(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>) -> JavaResult<JvmClassInstanceHandle<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:?})", &this);

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn set_color(jvm: &mut Jvm, _: &mut JavaContextArg, mut this: JvmClassInstanceHandle<Self>, rgb: i32) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {})", &this, rgb);

        jvm.put_field(&mut this, "rgb", "I", rgb)?;

        Ok(())
    }

    async fn set_color_by_rgb(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Graphics>,
        r: i32,
        g: i32,
        b: i32,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {}, {}, {})", &this, r, g, b);

        let rgb = (r << 16) | (g << 8) | b;

        jvm.put_field(&mut this, "rgb", "I", rgb)?;

        Ok(())
    }

    async fn set_font(
        _jvm: &mut Jvm,
        _: &mut JavaContextArg,
        this: JvmClassInstanceHandle<Graphics>,
        font: JvmClassInstanceHandle<Font>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setFont({:?}, {:?})", &this, &font);

        Ok(())
    }

    async fn set_alpha(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>, a1: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setAlpha({:?}, {})", &this, a1);

        Ok(())
    }

    async fn set_clip(
        _: &mut Jvm,
        _: &mut JavaContextArg,
        this: JvmClassInstanceHandle<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::setClip({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn clip_rect(
        _: &mut Jvm,
        _: &mut JavaContextArg,
        this: JvmClassInstanceHandle<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::clipRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn fill_rect(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I")?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

        canvas.fill_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_rect(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I")?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

        canvas.draw_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_string(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        string: JvmClassInstanceHandle<String>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawString({:?}, {:?}, {}, {}, {})",
            &this,
            &string,
            x,
            y,
            anchor.0
        );

        let rust_string = String::to_rust_string(jvm, &string)?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

        canvas.draw_text(&rust_string, x as _, y as _);

        Ok(())
    }

    async fn draw_line(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawLine({:?}, {}, {}, {}, {})", &this, x1, y1, x2, y2);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I")?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_image(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Self>,
        img: JvmClassInstanceHandle<Image>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:?}, {:?}, {}, {}, {})",
            &this,
            &img,
            x,
            y,
            anchor.0
        );

        let src_canvas = Image::image(jvm, &img)?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

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

        canvas.draw(x as _, y as _, src_canvas.width(), src_canvas.height(), &*src_canvas, 0, 0);

        Ok(())
    }

    async fn get_clip_x(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:?})", &this);

        Ok(0)
    }

    async fn get_clip_y(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:?})", &this);

        Ok(0)
    }

    async fn get_clip_width(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipWidth({:?})", &this);

        let w: i32 = jvm.get_field(&this, "w", "I")?;

        Ok(w)
    }

    async fn get_clip_height(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipHeight({:?})", &this);

        let h: i32 = jvm.get_field(&this, "h", "I")?;

        Ok(h)
    }

    async fn get_translate_x(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateX({:?})", &this);

        Ok(0)
    }

    async fn get_translate_y(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateY({:?})", &this);

        Ok(0)
    }

    async fn translate(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Graphics>, x: i32, y: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::translate({:?}, {}, {})", &this, x, y);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_rgb_pixels(
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        mut this: JvmClassInstanceHandle<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rgb_pixels: JvmClassInstanceHandle<Array<i32>>,
        offset: i32,
        bpl: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::setRGBPixels({:?}, {}, {}, {}, {}, {:?}, {}, {})",
            &this,
            x,
            y,
            width,
            height,
            &rgb_pixels,
            offset,
            bpl
        );

        // TODO we need imagebuffer proxy, as it's not optimal to copy entire image from java/c buffer to rust every time
        let pixel_data: Vec<i32> = jvm.load_array(&rgb_pixels, offset as _, (width * height) as _)?;
        let src_image = ImageBuffer::<Rgb8Pixel>::from_raw(width as _, height as _, cast_vec(pixel_data));

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image)?;

        canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0);

        Ok(())
    }

    async fn image(jvm: &mut Jvm, this: &mut JvmClassInstanceHandle<Graphics>) -> JavaResult<JvmClassInstanceHandle<Image>> {
        let image: JvmClassInstanceHandle<Image> = jvm.get_field(this, "img", "Lorg/kwis/msp/lcdui/Image;")?;

        if !image.is_null() {
            Ok(image)
        } else {
            let width = jvm.get_field(this, "w", "I")?;
            let height = jvm.get_field(this, "h", "I")?;

            let image: JvmClassInstanceHandle<Image> = jvm
                .invoke_static(
                    "org/kwis/msp/lcdui/Image",
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    [width, height],
                )
                .await?;

            jvm.put_field(this, "img", "Lorg/kwis/msp/lcdui/Image;", image.clone())?;

            Ok(image)
        }
    }
}
