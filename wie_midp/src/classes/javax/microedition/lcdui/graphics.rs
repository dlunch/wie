use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Image};

// class javax.microedition.lcdui.Graphics
pub struct Graphics;

impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, Default::default()),
                JavaMethodProto::new("drawRect", "(IIII)V", Self::draw_rect, Default::default()),
                JavaMethodProto::new("drawLine", "(IIII)V", Self::draw_line, Default::default()),
                JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, Default::default()),
                JavaMethodProto::new(
                    "drawImage",
                    "(Ljavax/microedition/lcdui/Image;III)V",
                    Self::draw_image,
                    Default::default(),
                ),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, Default::default()),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, Default::default()),
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_rgb, Default::default()),
                JavaMethodProto::new("reset", "()V", Self::reset, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn fill_rect(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::fillRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn draw_rect(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::drawRect({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        Ok(())
    }

    async fn draw_line(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::drawLine({:?}, {}, {}, {}, {})",
            &this,
            x1,
            y1,
            x2,
            y2
        );

        Ok(())
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
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::drawString({:?}, {:?}, {}, {}, {})",
            &this,
            string,
            x,
            y,
            anchor
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        tracing::warn!("draw_string: {:?}", string);

        Ok(())
    }

    async fn draw_image(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::drawImage({:?}, {:?}, {}, {}, {})",
            &this,
            &image,
            x,
            y,
            anchor
        );

        Ok(())
    }

    async fn set_font(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::setFont({:?}, {:?})", &this, font);

        Ok(())
    }

    async fn set_color(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::setColor({:?}, {})", &this, color);

        Ok(())
    }

    async fn set_color_rgb(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, red: i32, green: i32, blue: i32) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Graphics::setColor({:?}, {}, {}, {})",
            &this,
            red,
            green,
            blue
        );

        Ok(())
    }

    async fn reset(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::reset({:?})", &this);

        Ok(())
    }
}
