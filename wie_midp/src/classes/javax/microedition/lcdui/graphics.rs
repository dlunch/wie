use alloc::{string::String as RustString, vec, vec::Vec};

use bytemuck::cast_vec;

use jvm::{Array, ClassInstanceRef, JavaChar, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};

use java_class_proto::{JavaFieldProto, JavaMethodProto, TypeConverter};
use java_runtime::classes::java::lang::String;

use wie_backend::canvas::{Clip, PixelType, Rgb8Pixel, TextAlignment, VecImageBuffer};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Image};

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

impl From<Anchor> for TextAlignment {
    fn from(anchor: Anchor) -> Self {
        if anchor.contains(Anchor::HCENTER) {
            TextAlignment::Center
        } else if anchor.contains(Anchor::RIGHT) {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        }
    }
}

// class javax.microedition.lcdui.Graphics
pub struct Graphics;

#[allow(clippy::too_many_arguments)]
impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;)V", Self::init_with_image, Default::default()),
                JavaMethodProto::new("reset", "()V", Self::reset, Default::default()),
                JavaMethodProto::new("getFont", "()Ljavax/microedition/lcdui/Font;", Self::get_font, Default::default()),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, Default::default()),
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_by_rgb, Default::default()),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, Default::default()),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, Default::default()),
                JavaMethodProto::new("fillRoundRect", "(IIIIII)V", Self::fill_round_rect, Default::default()),
                JavaMethodProto::new("fillArc", "(IIIIII)V", Self::fill_arc, Default::default()),
                JavaMethodProto::new("drawLine", "(IIII)V", Self::draw_line, Default::default()),
                JavaMethodProto::new("drawRect", "(IIII)V", Self::draw_rect, Default::default()),
                JavaMethodProto::new("drawRoundRect", "(IIIIII)V", Self::draw_round_rect, Default::default()),
                JavaMethodProto::new("drawArc", "(IIIIII)V", Self::draw_arc, Default::default()),
                JavaMethodProto::new("drawChar", "(CIII)V", Self::draw_char, Default::default()),
                JavaMethodProto::new("drawChars", "([CIIIII)V", Self::draw_chars, Default::default()),
                JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, Default::default()),
                JavaMethodProto::new("drawSubstring", "(Ljava/lang/String;IIIII)V", Self::draw_substring, Default::default()),
                JavaMethodProto::new(
                    "drawImage",
                    "(Ljavax/microedition/lcdui/Image;III)V",
                    Self::draw_image,
                    Default::default(),
                ),
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
                JavaMethodProto::new("drawRGB", "([IIIIIIIZ)V", Self::draw_rgb, Default::default()),
                JavaMethodProto::new("setGrayScale", "(I)V", Self::set_gray_scale, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("img", "Ljavax/microedition/lcdui/Image;", Default::default()),
                JavaFieldProto::new("width", "I", Default::default()),
                JavaFieldProto::new("height", "I", Default::default()),
                JavaFieldProto::new("clipX", "I", Default::default()),
                JavaFieldProto::new("clipY", "I", Default::default()),
                JavaFieldProto::new("clipWidth", "I", Default::default()),
                JavaFieldProto::new("clipHeight", "I", Default::default()),
                JavaFieldProto::new("translateX", "I", Default::default()),
                JavaFieldProto::new("translateY", "I", Default::default()),
                JavaFieldProto::new("color", "I", Default::default()),
            ],
        }
    }

    async fn init_with_image(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::<init>({:?}, {:?})", &this, &image,);

        let width: i32 = jvm.invoke_virtual(&image, "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_virtual(&image, "getHeight", "()I", ()).await?;

        jvm.put_field(&mut this, "img", "Ljavax/microedition/lcdui/Image;", image).await?;

        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height).await?;

        Ok(())
    }

    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::reset({:?})", &this);

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        jvm.put_field(&mut this, "clipX", "I", 0).await?;
        jvm.put_field(&mut this, "clipY", "I", 0).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height).await?;
        jvm.put_field(&mut this, "translateX", "I", 0).await?;
        jvm.put_field(&mut this, "translateY", "I", 0).await?;
        jvm.put_field(&mut this, "color", "I", 0).await?;

        Ok(())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::getFont({:?})", &this);

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn set_color(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({:?}, {})", &this, rgb);

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_color_by_rgb(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({:?}, {}, {}, {})", &this, r, g, b);

        let rgb = (r << 16) | (g << 8) | b;

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_font(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::setFont({:?}, {:?})", &this, &font);

        Ok(())
    }

    async fn set_clip(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::setClip({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        jvm.put_field(&mut this, "clipX", "I", x).await?;
        jvm.put_field(&mut this, "clipY", "I", y).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height).await?;

        Ok(())
    }

    async fn clip_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::clipRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        let current_clip = Self::clip(jvm, &this).await?;
        let rect = Clip {
            x: x as _,
            y: y as _,
            width: width as _,
            height: height as _,
        };

        let new_clip = current_clip.intersect(&rect);

        jvm.put_field(&mut this, "clipX", "I", new_clip.x).await?;
        jvm.put_field(&mut this, "clipY", "I", new_clip.y).await?;
        jvm.put_field(&mut this, "clipWidth", "I", new_clip.width as i32).await?;
        jvm.put_field(&mut this, "clipHeight", "I", new_clip.height as i32).await?;

        Ok(())
    }

    async fn fill_round_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        arc_width: i32,
        arc_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::fillRoundRect({:?}, {}, {}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height,
            arc_width,
            arc_height
        );

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.fill_round_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            arc_width as _,
            arc_height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn fill_arc(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: i32,
        arc_angle: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::fillArc({:?}, {}, {}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height,
            start_angle,
            arc_angle
        );

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.fill_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            start_angle as _,
            arc_angle as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn fill_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        if x < 0 || y < 0 || width < 0 || height < 0 {
            tracing::warn!(
                "javax.microedition.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {}): invalid arguments",
                &this,
                x,
                y,
                width,
                height
            );
            return Ok(());
        }

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.fill_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn draw_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

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
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawChar({:?}, {}, {}, {}, {})",
            &this,
            ch,
            x,
            y,
            anchor.0
        );

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let string = RustString::from_utf16(&[ch]).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
        );

        Ok(())
    }

    async fn draw_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawChar({:?}, {:?}, {}, {}, {}, {})",
            &this,
            &chars,
            offset,
            length,
            x,
            y
        );

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let chars = jvm.load_array(&chars, offset as _, length as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
        );

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
            "javax.microedition.lcdui.Graphics::drawString({:?}, {:?}, {}, {}, {})",
            &this,
            &string,
            x,
            y,
            anchor.0
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
        );

        Ok(())
    }
    async fn draw_substring(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawSubstring({:?}, {:?}, {}, {}, {}, {})",
            &this,
            &string,
            offset,
            len,
            x,
            y
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        canvas.draw_text(
            &substring,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
        );

        Ok(())
    }

    async fn draw_line(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawLine({:?}, {}, {}, {}, {})", &this, x1, y1, x2, y2);

        let color: i32 = jvm.get_field(&this, "color", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x1 = x1 + translate_x;
        let y1 = y1 + translate_y;
        let x2 = x2 + translate_x;
        let y2 = y2 + translate_y;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(color as _));

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
            "javax.microedition.lcdui.Graphics::drawImage({:?}, {:?}, {}, {}, {})",
            &this,
            &img,
            x,
            y,
            anchor.0
        );

        if img.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

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

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x + x_delta;
        let y = translate_y + y + y_delta;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw(x as _, y as _, src_image.width(), src_image.height(), &*src_image, 0, 0, clip);

        Ok(())
    }

    async fn draw_round_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        arc_width: i32,
        arc_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRoundRect({:?}, {}, {}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height,
            arc_width,
            arc_height
        );

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_round_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            arc_width as _,
            arc_height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn draw_arc(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: i32,
        arc_angle: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawArc({:?}, {}, {}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height,
            start_angle,
            arc_angle
        );

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            start_angle as _,
            arc_angle as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn get_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getColor({:?})", &this);

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        Ok(color)
    }

    async fn get_clip_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipX({:?})", &this);

        let clip_x: i32 = jvm.get_field(&this, "clipX", "I").await?;

        Ok(clip_x)
    }

    async fn get_clip_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipY({:?})", &this);

        let clip_y: i32 = jvm.get_field(&this, "clipY", "I").await?;

        Ok(clip_y)
    }

    async fn get_clip_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipWidth({:?})", &this);

        let clip_width: i32 = jvm.get_field(&this, "clipWidth", "I").await?;

        Ok(clip_width)
    }

    async fn get_clip_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipHeight({:?})", &this);

        let clip_height: i32 = jvm.get_field(&this, "clipHeight", "I").await?;

        Ok(clip_height)
    }

    async fn get_translate_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateX({:?})", &this);

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;

        Ok(translate_x)
    }

    async fn get_translate_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateY({:?})", &this);

        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        Ok(translate_y)
    }

    async fn translate(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::translate({:?}, {}, {})", &this, x, y);

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        jvm.put_field(&mut this, "translateX", "I", translate_x + x).await?;
        jvm.put_field(&mut this, "translateY", "I", translate_y + y).await?;

        Ok(())
    }

    async fn draw_rgb(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        rgb_data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        scan_length: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        process_alpha: bool,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRGB({:?}, {:?}, {}, {}, {}, {}, {}, {}, {})",
            &this,
            &rgb_data,
            offset,
            scan_length,
            x,
            y,
            width,
            height,
            process_alpha
        );

        // TODO proper scanlength support
        let pixel_data: Vec<i32> = jvm.load_array(&rgb_data, offset as _, (width * height) as _).await?;
        let src_image = VecImageBuffer::<Rgb8Pixel>::from_raw(width as _, height as _, cast_vec(pixel_data));

        let image = Self::image(jvm, &mut this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0, clip);

        Ok(())
    }

    async fn set_gray_scale(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setGrayScale({:?}, {})", &this, value);

        let color = (value << 16) | (value << 8) | value;

        jvm.put_field(&mut this, "color", "I", color).await?;

        Ok(())
    }

    async fn image(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Image>> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "img", "Ljavax/microedition/lcdui/Image;").await?;

        if !image.is_null() {
            Ok(image)
        } else {
            let width = jvm.get_field(this, "width", "I").await?;
            let height = jvm.get_field(this, "height", "I").await?;

            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    [width, height],
                )
                .await?;

            jvm.put_field(this, "img", "Ljavax/microedition/lcdui/Image;", image.clone()).await?;

            Ok(image)
        }
    }

    async fn clip(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Clip> {
        let x: i32 = jvm.get_field(this, "clipX", "I").await?;
        let y: i32 = jvm.get_field(this, "clipY", "I").await?;
        let width: i32 = jvm.get_field(this, "clipWidth", "I").await?;
        let height: i32 = jvm.get_field(this, "clipHeight", "I").await?;

        Ok(Clip {
            x: x as _,
            y: y as _,
            width: width as _,
            height: height as _,
        })
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::lcdui::Image, get_protos};

    #[test]
    fn test_graphics() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (100, 100),
                )
                .await?;

            let graphics = jvm
                .new_class(
                    "javax/microedition/lcdui/Graphics",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    (image.clone(),),
                )
                .await?;

            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0x00ff00,)).await?;

            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let image = Image::image(&jvm, &image).await?;

            assert_eq!(image.width(), 100);
            assert_eq!(image.height(), 100);

            assert_eq!(image.colors()[0].r, 0x00);
            assert_eq!(image.colors()[0].g, 0xff);
            assert_eq!(image.colors()[0].b, 0x00);

            Ok(())
        })
    }
}
