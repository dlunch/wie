use alloc::{format, vec};

use jvm::{ClassInstanceRef, JavaValue};

use wie_backend::canvas::{PixelType, Rgb8Pixel};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JavaWord},
    method::TypeConverter,
    proxy::{JavaObjectProxy, JvmClassInstanceProxy},
    r#impl::{
        java::lang::String,
        org::kwis::msp::lcdui::{Display, Font, Image},
    },
    JavaFieldAccessFlag,
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
    fn to_rust(_: &mut dyn JavaContext, raw: JavaWord) -> Anchor {
        Anchor::from_bits_retain(raw as _)
    }

    fn from_rust(_: &mut dyn JavaContext, rust: Anchor) -> JavaWord {
        rust.bits() as _
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
            ],
            fields: vec![
                JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("h", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("rgb", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, display: JvmClassInstanceProxy<Display>) -> JavaResult<()> {
        let log = format!(
            "org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&display.class_instance)
        );
        tracing::debug!("{}", log); // splitted format as tracing macro doesn't like variable named `display` https://github.com/tokio-rs/tracing/issues/2332

        let width = context.jvm().get_field(&display.class_instance, "m_w", "I")?;
        let height = context.jvm().get_field(&display.class_instance, "m_h", "I")?;

        context.jvm().put_field(&this.class_instance, "w", "I", width)?;
        context.jvm().put_field(&this.class_instance, "h", "I", height)?;

        Ok(())
    }

    async fn init_with_image(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        image: JvmClassInstanceProxy<Image>,
        a0: i32,
        a1: i32,
        width: i32,
        height: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::<init>({:#x}, {:#x}, {}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&image.class_instance),
            a0,
            a1,
            width,
            height
        );

        context.jvm().put_field(
            &this.class_instance,
            "img",
            "Lorg/kwis/msp/lcdui/Image;",
            JavaValue::Object(Some(image.class_instance)),
        )?;
        context.jvm().put_field(&this.class_instance, "w", "I", JavaValue::Int(width as _))?;
        context.jvm().put_field(&this.class_instance, "h", "I", JavaValue::Int(height as _))?;

        Ok(())
    }

    async fn get_font(context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<JvmClassInstanceProxy<Font>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getFont({:#x})", this.ptr_instance);

        let font = context.jvm().instantiate_class("org/kwis/msp/lcdui/Font").await?;
        context
            .jvm()
            .invoke_method(&font, "org/kwis/msp/lcdui/Font", "<init>", "()V", &[])
            .await?;

        Ok(JvmClassInstanceProxy::new(font))
    }

    async fn set_color(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, rgb: i32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::setColor({:#x}, {})",
            context.instance_raw(&this.class_instance),
            rgb
        );

        context.jvm().put_field(&this.class_instance, "rgb", "I", JavaValue::Int(rgb as _))?;

        Ok(())
    }

    async fn set_color_by_rgb(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Graphics>, r: i32, g: i32, b: i32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::setColor({:#x}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            r,
            g,
            b
        );

        let rgb = (r << 16) | (g << 8) | b;

        context.jvm().put_field(&this.class_instance, "rgb", "I", JavaValue::Int(rgb as _))?;

        Ok(())
    }

    async fn set_font(_context: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, font: JavaObjectProxy<Font>) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::setFont({:#x}, {:#x})",
            this.ptr_instance,
            font.ptr_instance
        );

        Ok(())
    }

    async fn set_alpha(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, a1: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::setAlpha({:#x}, {})", this.ptr_instance, a1);

        Ok(())
    }

    async fn set_clip(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, x: i32, y: i32, width: i32, height: i32) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::setClip({:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn clip_rect(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, x: i32, y: i32, width: i32, height: i32) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::clipRect({:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn fill_rect(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, x: i32, y: i32, width: i32, height: i32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::fillRect({:#x}, {}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            x,
            y,
            width,
            height
        );

        let rgb = context.jvm().get_field(&this.class_instance, "rgb", "I")?.as_int();

        let image = Self::image(context, &this.class_instance).await?;
        let mut canvas = Image::canvas(context, &image.class_instance)?;

        canvas.fill_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_rect(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, x: i32, y: i32, width: i32, height: i32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawRect({:#x}, {}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            x,
            y,
            width,
            height
        );

        let rgb = context.jvm().get_field(&this.class_instance, "rgb", "I")?.as_int();

        let image = Self::image(context, &this.class_instance).await?;
        let mut canvas = Image::canvas(context, &image.class_instance)?;

        canvas.draw_rect(x as _, y as _, width as _, height as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_string(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        string: JvmClassInstanceProxy<String>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawString({:#x}, {:#x}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&string.class_instance),
            x,
            y,
            anchor.0
        );

        let rust_string = String::to_rust_string(context, &string.class_instance)?;

        let image = Self::image(context, &this.class_instance).await?;
        let mut canvas = Image::canvas(context, &image.class_instance)?;

        canvas.draw_text(&rust_string, x as _, y as _);

        Ok(())
    }

    async fn draw_line(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawLine({:#x}, {}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            x1,
            y1,
            x2,
            y2
        );

        let rgb = context.jvm().get_field(&this.class_instance, "rgb", "I")?.as_int();

        let image = Self::image(context, &this.class_instance).await?;
        let mut canvas = Image::canvas(context, &image.class_instance)?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(rgb as _));

        Ok(())
    }

    async fn draw_image(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        img: JvmClassInstanceProxy<Image>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:#x}, {:#x}, {}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&img.class_instance),
            x,
            y,
            anchor.0
        );

        let src_canvas = Image::image(context, &img.class_instance)?;

        let image = Self::image(context, &this.class_instance).await?;
        let mut canvas = Image::canvas(context, &image.class_instance)?;

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

    async fn get_clip_x(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipX({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_y(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getClipY({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_clip_width(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::getClipWidth({:#x})",
            context.instance_raw(&this.class_instance)
        );

        let w = context.jvm().get_field(&this.class_instance, "w", "I")?.as_int();

        Ok(w as _)
    }

    async fn get_clip_height(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Graphics::getClipHeight({:#x})",
            context.instance_raw(&this.class_instance)
        );

        let h = context.jvm().get_field(&this.class_instance, "h", "I")?.as_int();

        Ok(h as _)
    }

    async fn get_translate_x(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateX({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn get_translate_y(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::getTranslateY({:#x})", this.ptr_instance);

        Ok(0)
    }

    async fn translate(_: &mut dyn JavaContext, this: JavaObjectProxy<Graphics>, x: i32, y: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Graphics::translate({:#x}, {}, {})", this.ptr_instance, x, y);

        Ok(())
    }

    async fn image(context: &mut dyn JavaContext, this: &ClassInstanceRef) -> JavaResult<JvmClassInstanceProxy<Image>> {
        let image = context.jvm().get_field(this, "img", "Lorg/kwis/msp/lcdui/Image;")?;

        if image.as_object_ref().is_some() {
            Ok(JvmClassInstanceProxy::new(image.as_object_ref().unwrap().clone()))
        } else {
            let width = context.jvm().get_field(this, "w", "I")?;
            let height = context.jvm().get_field(this, "h", "I")?;

            let image = context
                .jvm()
                .invoke_static_method(
                    "org/kwis/msp/lcdui/Image",
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    &[width, height],
                )
                .await?;

            context.jvm().put_field(
                this,
                "img",
                "Lorg/kwis/msp/lcdui/Image;",
                JavaValue::Object(Some(image.as_object_ref().unwrap().clone())),
            )?;

            Ok(JvmClassInstanceProxy::new(image.as_object().unwrap()))
        }
    }
}
