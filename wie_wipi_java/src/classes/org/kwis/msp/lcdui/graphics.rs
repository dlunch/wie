use alloc::{format, string::String as RustString, vec, vec::Vec};

use bytemuck::cast_vec;
use jvm::{runtime::JavaLangString, JavaChar, JavaValue};

use wie_backend::canvas::{PixelType, Rgb8Pixel, TextAlignment, VecImageBuffer};

use java_class_proto::{JavaFieldProto, JavaMethodProto, TypeConverter};
use java_runtime::classes::java::lang::String;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::{Display, Font, Image};

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
    fn to_rust(_: &Jvm, raw: JavaValue) -> Anchor {
        let raw: i32 = raw.into();
        Anchor::from_bits_retain(raw)
    }

    fn from_rust(_: &Jvm, rust: Anchor) -> JavaValue {
        rust.bits().into()
    }
}

// class org.kwis.msp.lcdui.Graphics
pub struct Graphics;

impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Image;IIII)V", Self::init_with_image, Default::default()),
                JavaMethodProto::new("getFont", "()Lorg/kwis/msp/lcdui/Font;", Self::get_font, Default::default()),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, Default::default()),
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_by_rgb, Default::default()),
                JavaMethodProto::new("setFont", "(Lorg/kwis/msp/lcdui/Font;)V", Self::set_font, Default::default()),
                JavaMethodProto::new("setAlpha", "(I)V", Self::set_alpha, Default::default()),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, Default::default()),
                JavaMethodProto::new("drawLine", "(IIII)V", Self::draw_line, Default::default()),
                JavaMethodProto::new("drawRect", "(IIII)V", Self::draw_rect, Default::default()),
                JavaMethodProto::new("drawChar", "(CIII)V", Self::draw_char, Default::default()),
                JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, Default::default()),
                JavaMethodProto::new("drawImage", "(Lorg/kwis/msp/lcdui/Image;III)V", Self::draw_image, Default::default()),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, Default::default()),
                JavaMethodProto::new("clipRect", "(IIII)V", Self::clip_rect, Default::default()),
                JavaMethodProto::new("getColor", "()I", Self::get_color, Default::default()),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x, Default::default()),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y, Default::default()),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width, Default::default()),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height, Default::default()),
                JavaMethodProto::new("getTranslateX", "()I", Self::get_translate_x, Default::default()),
                JavaMethodProto::new("getTranslateY", "()I", Self::get_translate_y, Default::default()),
                JavaMethodProto::new("translate", "(II)V", Self::translate, Default::default()),
                JavaMethodProto::new("setRGBPixels", "(IIII[III)V", Self::set_rgb_pixels, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", Default::default()),
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
                JavaFieldProto::new("rgb", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, display: ClassInstanceRef<Display>) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Graphics::<init>({:?}, {:?})", &this, &display);
        tracing::debug!("{}", log); // splitted format as tracing macro doesn't like variable named `display` https://github.com/tokio-rs/tracing/issues/2332

        let width: i32 = jvm.get_field(&display, "m_w", "I").await?;
        let height: i32 = jvm.get_field(&display, "m_h", "I").await?;

        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        a0: i32,
        a1: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::<init>({:?}, {:?}, {}, {}, {}, {})",
            &this,
            &image,
            a0,
            a1,
            width,
            height
        );

        jvm.put_field(&mut this, "img", "Lorg/kwis/msp/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;

        Ok(())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:?})", &this);

        let instance = jvm.new_class("org/kwis/msp/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn set_color(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {})", &this, rgb);

        jvm.put_field(&mut this, "rgb", "I", rgb).await?;

        Ok(())
    }

    async fn set_color_by_rgb(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {}, {}, {})", &this, r, g, b);

        let rgb = (r << 16) | (g << 8) | b;

        jvm.put_field(&mut this, "rgb", "I", rgb).await?;

        Ok(())
    }

    async fn set_font(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setFont({:?}, {:?})", &this, &font);

        Ok(())
    }

    async fn set_alpha(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, a1: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setAlpha({:?}, {})", &this, a1);

        Ok(())
    }

    async fn set_clip(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
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

    async fn clip_rect(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
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

    async fn fill_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        if x < 0 || y < 0 || width < 0 || height < 0 {
            tracing::warn!(
                "org.kwis.msp.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {}): invalid arguments",
                &this,
                x,
                y,
                width,
                height
            );
            return Ok(());
        }

        let rgb: i32 = jvm.get_field(&this, "rgb", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.fill_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        canvas.flush().await;

        Ok(())
    }

    async fn draw_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.draw_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        canvas.flush().await;

        Ok(())
    }

    async fn draw_char(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        ch: JavaChar,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawChar({:?}, {}, {}, {}, {})", &this, ch, x, y, anchor.0);

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let alignment = if anchor.contains(Anchor::HCENTER) {
            TextAlignment::Center
        } else if anchor.contains(Anchor::RIGHT) {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        };

        let string = RustString::from_utf16(&[ch]).unwrap();

        canvas.draw_text(&string, x as _, y as _, alignment);

        canvas.flush().await;

        Ok(())
    }

    async fn draw_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawString({:?}, {:?}, {}, {}, {})",
            &this,
            &string,
            x,
            y,
            anchor.0
        );

        let rust_string = JavaLangString::to_rust_string(jvm, &string).await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let alignment = if anchor.contains(Anchor::HCENTER) {
            TextAlignment::Center
        } else if anchor.contains(Anchor::RIGHT) {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        };

        canvas.draw_text(&rust_string, x as _, y as _, alignment);

        canvas.flush().await;

        Ok(())
    }

    async fn draw_line(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawLine({:?}, {}, {}, {}, {})", &this, x1, y1, x2, y2);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(rgb as _));

        canvas.flush().await;

        Ok(())
    }

    async fn draw_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        img: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:?}, {:?}, {}, {}, {})",
            &this,
            &img,
            x,
            y,
            anchor.0
        );

        let src_image = Image::image(jvm, &img).await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let x_delta = if anchor.contains(Anchor::HCENTER) {
            -((src_image.width() / 2) as i32)
        } else if anchor.contains(Anchor::RIGHT) {
            -(src_image.width() as i32)
        } else {
            0
        };

        let y_delta = if anchor.contains(Anchor::VCENTER) {
            -((src_image.height() / 2) as i32)
        } else if anchor.contains(Anchor::BOTTOM) {
            -(src_image.height() as i32)
        } else {
            0
        };

        let x = (x + x_delta).max(0);
        let y = (y + y_delta).max(0);

        canvas.draw(x as _, y as _, src_image.width(), src_image.height(), &*src_image, 0, 0);

        canvas.flush().await;

        Ok(())
    }

    async fn get_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getColor({:?})", &this);

        let rgb: i32 = jvm.get_field(&this, "rgb", "I").await?;

        Ok(rgb)
    }

    async fn get_clip_x(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:?})", &this);

        Ok(0)
    }

    async fn get_clip_y(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:?})", &this);

        Ok(0)
    }

    async fn get_clip_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipWidth({:?})", &this);

        let w: i32 = jvm.get_field(&this, "w", "I").await?;

        Ok(w)
    }

    async fn get_clip_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipHeight({:?})", &this);

        let h: i32 = jvm.get_field(&this, "h", "I").await?;

        Ok(h)
    }

    async fn get_translate_x(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateX({:?})", &this);

        Ok(0)
    }

    async fn get_translate_y(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateY({:?})", &this);

        Ok(0)
    }

    async fn translate(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, x: i32, y: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::translate({:?}, {}, {})", &this, x, y);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_rgb_pixels(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        rgb_pixels: ClassInstanceRef<Array<i32>>,
        offset: i32,
        bpl: i32,
    ) -> JvmResult<()> {
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
        let pixel_data: Vec<i32> = jvm.load_array(&rgb_pixels, offset as _, (width * height) as _).await?;
        let src_image = VecImageBuffer::<Rgb8Pixel>::from_raw(width as _, height as _, cast_vec(pixel_data));

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0);

        canvas.flush().await;

        Ok(())
    }

    async fn image(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Image>> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "img", "Lorg/kwis/msp/lcdui/Image;").await?;

        if !image.is_null() {
            Ok(image)
        } else {
            let width = jvm.get_field(this, "w", "I").await?;
            let height = jvm.get_field(this, "h", "I").await?;

            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "org/kwis/msp/lcdui/Image",
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    [width, height],
                )
                .await?;

            jvm.put_field(this, "img", "Lorg/kwis/msp/lcdui/Image;", image.clone()).await?;

            Ok(image)
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::lcdui::Image, get_protos};

    #[test]
    fn test_graphics() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static("org/kwis/msp/lcdui/Image", "createImage", "(II)Lorg/kwis/msp/lcdui/Image;", (100, 100))
                .await?;

            let graphics = jvm
                .new_class(
                    "org/kwis/msp/lcdui/Graphics",
                    "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                    (image.clone(), 0, 0, 100, 100),
                )
                .await?;

            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0x00ff00,)).await?;

            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let image = Image::image(&jvm, &image).await?;

            assert_eq!(image.width(), 100);
            assert_eq!(image.height(), 100);

            assert_eq!(image.raw()[0], 0);
            assert_eq!(image.raw()[1], 255);
            assert_eq!(image.raw()[2], 0);

            Ok(())
        })
    }
}
