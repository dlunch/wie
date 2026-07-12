use alloc::{boxed::Box, string::String as RustString, vec, vec::Vec};

use bytemuck::cast_vec;

use jvm::{Array, ClassInstanceRef, JavaChar, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};

use java_class_proto::{JavaFieldProto, JavaMethodProto, TypeConverter};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;

use wie_backend::canvas::{ArgbPixel, Canvas as BackendCanvas, Clip, PixelType, Rgb8Pixel, TextAlignment, VecImageBuffer};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Image};

bitflags::bitflags! {
    struct Anchor: i32 {
        const HCENTER = 1;
        const VCENTER = 2;
        const LEFT = 4;
        const RIGHT = 8;
        const TOP = 16;
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
                // WIPI wrapper bridge only; this is not part of the MIDP Graphics API.
                JavaMethodProto::new("setXORMode", "(Z)V", Self::set_xor_mode, MethodAccessFlags::PRIVATE),
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
                JavaMethodProto::new(
                    "drawRegion",
                    "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
                    Self::draw_region,
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
                JavaFieldProto::new("xorMode", "Z", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init_with_image(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::<init>({this:?}, {image:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let width: i32 = jvm.invoke_virtual(&image, "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_virtual(&image, "getHeight", "()I", ()).await?;

        jvm.put_field(&mut this, "img", "Ljavax/microedition/lcdui/Image;", image).await?;

        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        let _: () = jvm.invoke_virtual(&this, "reset", "()V", ()).await?;

        Ok(())
    }

    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::reset({this:?})");

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        jvm.put_field(&mut this, "clipX", "I", 0).await?;
        jvm.put_field(&mut this, "clipY", "I", 0).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height).await?;
        jvm.put_field(&mut this, "translateX", "I", 0).await?;
        jvm.put_field(&mut this, "translateY", "I", 0).await?;
        jvm.put_field(&mut this, "color", "I", 0).await?;
        jvm.put_field(&mut this, "xorMode", "Z", false).await?;

        Ok(())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::getFont({this:?})");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn set_color(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({this:?}, {rgb})");

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_color_by_rgb(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({this:?}, {r}, {g}, {b})");

        let rgb = (r << 16) | (g << 8) | b;

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_xor_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, xor_mode: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setXORMode({this:?}, {xor_mode})");

        jvm.put_field(&mut this, "xorMode", "Z", xor_mode).await?;

        Ok(())
    }

    async fn set_font(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::setFont({this:?}, {font:?})");

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
        tracing::debug!("javax.microedition.lcdui.Graphics::setClip({this:?}, {x}, {y}, {width}, {height})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        // clip fields hold absolute coordinates; negative w/h must clamp to 0 or `Self::clip()`'s
        // u32 cast produces a huge clip that copy_area's i64 extension treats as unbounded
        jvm.put_field(&mut this, "clipX", "I", x + translate_x).await?;
        jvm.put_field(&mut this, "clipY", "I", y + translate_y).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width.max(0)).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height.max(0)).await?;

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
        tracing::debug!("javax.microedition.lcdui.Graphics::clipRect({this:?}, {x}, {y}, {width}, {height})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let current_clip = Self::clip(jvm, &this).await?;
        let rect = Clip {
            x: x + translate_x,
            y: y + translate_y,
            width: width.max(0) as _,
            height: height.max(0) as _,
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
        tracing::debug!("javax.microedition.lcdui.Graphics::fillRoundRect({this:?}, {x}, {y}, {width}, {height}, {arc_width}, {arc_height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

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
        tracing::debug!("javax.microedition.lcdui.Graphics::fillArc({this:?}, {x}, {y}, {width}, {height}, {start_angle}, {arc_angle})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        canvas.fill_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            start_angle,
            arc_angle,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn fill_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::fillRect({this:?}, {x}, {y}, {width}, {height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawRect({this:?}, {x}, {y}, {width}, {height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawRect outlines an area width+1 pixels wide and height+1 tall
        canvas.draw_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawChar({this:?}, {ch}, {x}, {y}, {})", anchor.0);

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let string = RustString::from_utf16(&[ch]).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawChar({this:?}, {chars:?}, {offset}, {length}, {x}, {y})");

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let chars = jvm.load_array(&chars, offset as _, length as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
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
            "javax.microedition.lcdui.Graphics::drawString({this:?}, {string:?}, {x}, {y}, {})",
            anchor.0
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawSubstring({this:?}, {string:?}, {offset}, {len}, {x}, {y})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            &substring,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
        );

        Ok(())
    }

    async fn draw_line(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawLine({this:?}, {x1}, {y1}, {x2}, {y2})");

        let color: i32 = jvm.get_field(&this, "color", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x1 = x1 + translate_x;
        let y1 = y1 + translate_y;
        let x2 = x2 + translate_x;
        let y2 = y2 + translate_y;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(color as _), clip);

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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawImage({this:?}, {img:?}, {x}, {y}, {})", anchor.0);

        if img.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let src_image = Image::image(jvm, &img).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

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

    async fn draw_region(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        img: ClassInstanceRef<Image>,
        src_x: i32,
        src_y: i32,
        width: i32,
        height: i32,
        transform: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRegion({this:?}, {img:?}, {src_x}, {src_y}, {width}, {height}, {transform}, {x}, {y}, {})",
            anchor.0
        );

        if img.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let src_image = Image::image(jvm, &img).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let x_delta = if anchor.contains(Anchor::HCENTER) {
            -width / 2
        } else if anchor.contains(Anchor::RIGHT) {
            -height
        } else {
            0
        };

        let y_delta = if anchor.contains(Anchor::VCENTER) {
            -height / 2
        } else if anchor.contains(Anchor::BOTTOM) {
            -height
        } else {
            0
        };

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x + x_delta;
        let y = translate_y + y + y_delta;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw(x as _, y as _, width as _, height as _, &*src_image, src_x, src_y, clip);

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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawRoundRect({this:?}, {x}, {y}, {width}, {height}, {arc_width}, {arc_height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawRoundRect outlines an area width+1 pixels wide and height+1 tall
        canvas.draw_round_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
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
        tracing::debug!("javax.microedition.lcdui.Graphics::drawArc({this:?}, {x}, {y}, {width}, {height}, {start_angle}, {arc_angle})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawArc covers an area width+1 pixels wide and height+1 tall
        canvas.draw_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
            start_angle,
            arc_angle,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn get_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getColor({this:?})");

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        Ok(color)
    }

    async fn get_clip_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipX({this:?})");

        let clip_x: i32 = jvm.get_field(&this, "clipX", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;

        Ok(clip_x - translate_x)
    }

    async fn get_clip_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipY({this:?})");

        let clip_y: i32 = jvm.get_field(&this, "clipY", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        Ok(clip_y - translate_y)
    }

    async fn get_clip_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipWidth({this:?})");

        let clip_width: i32 = jvm.get_field(&this, "clipWidth", "I").await?;

        Ok(clip_width)
    }

    async fn get_clip_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipHeight({this:?})");

        let clip_height: i32 = jvm.get_field(&this, "clipHeight", "I").await?;

        Ok(clip_height)
    }

    async fn get_translate_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateX({this:?})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;

        Ok(translate_x)
    }

    async fn get_translate_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateY({this:?})");

        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        Ok(translate_y)
    }

    async fn translate(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::translate({this:?}, {x}, {y})");

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
            "javax.microedition.lcdui.Graphics::drawRGB({this:?}, {rgb_data:?}, {offset}, {scan_length}, {x}, {y}, {width}, {height}, {process_alpha})"
        );

        // TODO proper scanlength support
        let pixel_data: Vec<i32> = jvm.load_array(&rgb_data, offset as _, (width * height) as _).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x;
        let y = translate_y + y;

        let clip = Self::clip(jvm, &this).await?;

        if process_alpha {
            let src_image = VecImageBuffer::<ArgbPixel>::from_raw(width as _, height as _, cast_vec(pixel_data));
            canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0, clip);
        } else {
            let src_image = VecImageBuffer::<Rgb8Pixel>::from_raw(width as _, height as _, cast_vec(pixel_data));
            canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0, clip);
        }

        Ok(())
    }

    async fn set_gray_scale(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setGrayScale({this:?}, {value})");

        let color = (value << 16) | (value << 8) | value;

        jvm.put_field(&mut this, "color", "I", color).await?;

        Ok(())
    }

    async fn canvas(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<Box<dyn BackendCanvas>> {
        let image = Self::image(jvm, this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        let xor_mode: bool = jvm.get_field(this, "xorMode", "Z").await?;

        canvas.set_xor_mode(xor_mode);

        Ok(canvas)
    }

    pub async fn image(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Image>> {
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

    pub async fn clip(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Clip> {
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
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstance, ClassInstanceRef, Jvm, Result as JvmResult};

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

            let backend_image = Image::image(&jvm, &image).await?;

            assert_eq!(backend_image.width(), 100);
            assert_eq!(backend_image.height(), 100);

            assert_eq!(backend_image.colors()[0].r, 0x00);
            assert_eq!(backend_image.colors()[0].g, 0xff);
            assert_eq!(backend_image.colors()[0].b, 0x00);

            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0x123456,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xf00faa,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setXORMode", "(Z)V", (true,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(0, 0);
            assert_eq!(color.r, 0x12 ^ 0xf0);
            assert_eq!(color.g, 0x34 ^ 0x0f);
            assert_eq!(color.b, 0x56 ^ 0xaa);

            let _: () = jvm.invoke_virtual(&graphics, "setXORMode", "(Z)V", (false,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0x123456,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setXORMode", "(Z)V", (true,)).await?;

            let mut rgb_data = jvm.instantiate_array("I", 1).await?;
            jvm.store_array(&mut rgb_data, 0, vec![0x00ff0000i32]).await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "drawRGB", "([IIIIIIIZ)V", (rgb_data, 0, 1, 0, 0, 1, 1, true))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(0, 0);
            assert_eq!(color.r, 0x12);
            assert_eq!(color.g, 0x34);
            assert_eq!(color.b, 0x56);

            Ok(())
        })
    }

    async fn new_graphics(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<Image>, Box<dyn ClassInstance>)> {
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

        Ok((image, graphics))
    }

    #[test]
    fn test_clip_follows_translate() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (10, 10)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, 5, 5)).await?;

            let clip_x: i32 = jvm.invoke_virtual(&graphics, "getClipX", "()I", ()).await?;
            let clip_y: i32 = jvm.invoke_virtual(&graphics, "getClipY", "()I", ()).await?;
            assert_eq!(clip_x, 0);
            assert_eq!(clip_y, 0);

            let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (-10, -10)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xff0000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(10, 10), (14, 14)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            }
            for (x, y) in [(9, 9), (15, 15)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_clip_rect_intersects_in_translated_coords() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, 20, 20)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (10, 10)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "clipRect", "(IIII)V", (0, 0, 5, 5)).await?;

            let clip_x: i32 = jvm.invoke_virtual(&graphics, "getClipX", "()I", ()).await?;
            let clip_y: i32 = jvm.invoke_virtual(&graphics, "getClipY", "()I", ()).await?;
            let clip_width: i32 = jvm.invoke_virtual(&graphics, "getClipWidth", "()I", ()).await?;
            let clip_height: i32 = jvm.invoke_virtual(&graphics, "getClipHeight", "()I", ()).await?;
            assert_eq!(clip_x, 0);
            assert_eq!(clip_y, 0);
            assert_eq!(clip_width, 5);
            assert_eq!(clip_height, 5);

            let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (-10, -10)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xff0000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(10, 10);
            assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            for (x, y) in [(9, 9), (15, 15)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_empty_clip_draws_nothing() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, 5, 5)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "clipRect", "(IIII)V", (20, 20, 5, 5)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xff0000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(0, 0), (2, 2), (21, 21)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_negative_clip_is_empty() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, -5, -5)).await?;

            let clip_width: i32 = jvm.invoke_virtual(&graphics, "getClipWidth", "()I", ()).await?;
            let clip_height: i32 = jvm.invoke_virtual(&graphics, "getClipHeight", "()I", ()).await?;
            assert_eq!(clip_width, 0);
            assert_eq!(clip_height, 0);

            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xff0000,)).await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, 100, 100)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(0, 0), (50, 50)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_draw_rect_is_inclusive() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0xff0000,)).await?;
            // MIDP: drawRect(2, 2, 0, 3) draws a 1px-wide, 4px-tall vertical line
            let _: () = jvm.invoke_virtual(&graphics, "drawRect", "(IIII)V", (2, 2, 0, 3)).await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for y in 2..=5 {
                let color = backend_image.get_pixel(2, y);
                assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00), "y={y}");
            }
            let color = backend_image.get_pixel(2, 6);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            let color = backend_image.get_pixel(3, 2);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));

            Ok(())
        })
    }

    #[test]
    fn test_draw_rgb_follows_translate() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (10, 10)).await?;

            let mut rgb_data = jvm.instantiate_array("I", 1).await?;
            jvm.store_array(&mut rgb_data, 0, vec![0x00ff0000i32]).await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "drawRGB", "([IIIIIIIZ)V", (rgb_data, 0, 1, 0, 0, 1, 1, false))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(10, 10);
            assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            let color = backend_image.get_pixel(0, 0);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));

            Ok(())
        })
    }
}
