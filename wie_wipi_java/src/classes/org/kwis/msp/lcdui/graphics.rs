use alloc::vec;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Display as MidpDisplay, Font as MidpFont, Graphics as MidpGraphics, Image as MidpImage};

use crate::classes::org::kwis::msp::lcdui::{Display, Font, Image};

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
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::init_with_midp_graphics,
                    Default::default(),
                ),
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
            fields: vec![JavaFieldProto::new(
                "midpGraphics",
                "Ljavax/microedition/lcdui/Graphics;",
                Default::default(),
            )],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, display: ClassInstanceRef<Display>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::<init>({:?})", &this);

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&display, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let midp_graphics = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Display;)V",
                (midp_display,),
            )
            .await?;

        jvm.put_field(&mut this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;", midp_graphics)
            .await?;

        Ok(())
    }

    async fn init_with_midp_graphics(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        midp_graphics: ClassInstanceRef<MidpGraphics>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::<init>({:?})", &this);

        jvm.put_field(&mut this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;", midp_graphics)
            .await?;

        Ok(())
    }

    async fn get_font(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getFont({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .invoke_virtual(&midp_graphics, "getFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;

        Ok(jvm
            .new_class("org/kwis/msp/lcdui/Font", "(Ljavax/microedition/lcdui/Font;)V", (midp_font,))
            .await?
            .into())
    }

    async fn set_color(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {})", &this, color);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "setColor", "(I)V", (color,)).await
    }

    async fn set_color_by_rgb(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setColor({:?}, {}, {}, {})", &this, r, g, b);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "setColor", "(III)V", (r, g, b)).await
    }

    async fn set_font(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setFont({:?}, {:?})", &this, &font);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        let midp_font: ClassInstanceRef<MidpFont> = jvm.get_field(&font, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;

        jvm.invoke_virtual(&midp_graphics, "setFont", "(Ljavax/microedition/lcdui/Font;)V", (midp_font,))
            .await
    }

    async fn set_alpha(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, alpha: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setAlpha({:?}, {})", &this, alpha);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "setAlpha", "(I)V", (alpha,)).await
    }

    async fn fill_rect(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "fillRect", "(IIII)V", (x, y, width, height)).await
    }

    async fn draw_line(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawLine({:?}, {}, {}, {}, {})", &this, x1, y1, x2, y2);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "drawLine", "(IIII)V", (x1, y1, x2, y2)).await
    }

    async fn draw_rect(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "drawRect", "(IIII)V", (x, y, width, height)).await
    }

    async fn draw_char(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        ch: JavaChar,
        x: i32,
        y: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::drawChar({:?}, {:?}, {}, {}, {})", &this, ch, x, y, anchor);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "drawChar", "(CIII)V", (ch, x, y, anchor)).await
    }

    async fn draw_string(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        x: i32,
        y: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawString({:?}, {:?}, {}, {}, {})",
            &this,
            &string,
            x,
            y,
            anchor
        );

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;

        jvm.invoke_virtual(&midp_graphics, "drawString", "(Ljava/lang/String;III)V", (string, x, y, anchor))
            .await
    }

    async fn draw_image(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Graphics::drawImage({:?}, {:?}, {}, {}, {})",
            &this,
            &image,
            x,
            y,
            anchor
        );

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&image, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        jvm.invoke_virtual(
            &midp_graphics,
            "drawImage",
            "(Ljavax/microedition/lcdui/Image;III)V",
            (midp_image, x, y, anchor),
        )
        .await
    }

    async fn set_clip(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::setClip({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "setClip", "(IIII)V", (x, y, width, height)).await
    }

    async fn clip_rect(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::clipRect({:?}, {}, {}, {}, {})", &this, x, y, width, height);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "clipRect", "(IIII)V", (x, y, width, height)).await
    }

    async fn get_color(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getColor({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getColor", "()I", ()).await
    }

    async fn get_clip_x(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getClipX({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getClipX", "()I", ()).await
    }

    async fn get_clip_y(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getClipY({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getClipY", "()I", ()).await
    }

    async fn get_clip_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getClipWidth({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getClipWidth", "()I", ()).await
    }

    async fn get_clip_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getClipHeight({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getClipHeight", "()I", ()).await
    }

    async fn get_translate_x(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getTranslateX({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getTranslateX", "()I", ()).await
    }

    async fn get_translate_y(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::getTranslateY({:?})", &this);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "getTranslateY", "()I", ()).await
    }

    async fn translate(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Graphics::translate({:?}, {}, {})", &this, x, y);

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        jvm.invoke_virtual(&midp_graphics, "translate", "(II)V", (x, y)).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_rgb_pixels(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
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

        let midp_graphics = jvm.get_field(&this, "midpGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;

        jvm.invoke_virtual(
            &midp_graphics,
            "drawRGB",
            "([IIIIIIIZ)V",
            (rgb_pixels, offset, bpl, x, y, width, height, true),
        )
        .await
    }
}
