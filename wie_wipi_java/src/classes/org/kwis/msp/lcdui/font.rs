use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Font as MidpFont;

// class org.kwis.msp.lcdui.Font
pub struct Font;

impl Font {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Font;)V", Self::init, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Lorg/kwis/msp/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getFont", "(III)Lorg/kwis/msp/lcdui/Font;", Self::get_font, MethodAccessFlags::STATIC),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, Default::default()),
                JavaMethodProto::new("substringWidth", "(Ljava/lang/String;II)I", Self::substring_width, Default::default()),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, Default::default()),
                JavaMethodProto::new("charsWidth", "([CII)I", Self::chars_width, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("midpFont", "Ljavax/microedition/lcdui/Font;", Default::default()),
                JavaFieldProto::new("FACE_MONOSPACE", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
            ],
            access_flags: Default::default(),
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<clinit>");

        let face_monospace: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I").await?;
        let face_system: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I").await?;
        let style_plain: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I").await?;
        let size_small: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I").await?;

        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_MONOSPACE", "I", face_monospace)
            .await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_SYSTEM", "I", face_system).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_PLAIN", "I", style_plain).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_SMALL", "I", size_small).await?;

        Ok(())
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, midp_font: ClassInstanceRef<MidpFont>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<init>({:?})", &this);

        jvm.put_field(&mut this, "midpFont", "Ljavax/microedition/lcdui/Font;", midp_font).await?;

        Ok(())
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getHeight");

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "getHeight", "()I", ()).await
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getDefaultFont");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;

        Ok(jvm
            .new_class("org/kwis/msp/lcdui/Font", "(Ljavax/microedition/lcdui/Font;)V", (midp_font,))
            .await?
            .into())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getFont({:?}, {:?}, {:?})", face, style, size);

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Font",
                "getFont",
                "(III)Ljavax/microedition/lcdui/Font;",
                (face, style, size),
            )
            .await?;

        Ok(jvm
            .new_class("org/kwis/msp/lcdui/Font", "(Ljavax/microedition/lcdui/Font;)V", (midp_font,))
            .await?
            .into())
    }

    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::stringWidth({:?})", &string);

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "stringWidth", "(Ljava/lang/String;)I", (string,)).await
    }

    async fn substring_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::substringWidth({:?}, {:?}, {:?})", &string, offset, len);

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "substringWidth", "(Ljava/lang/String;II)I", (string, offset, len))
            .await
    }

    async fn char_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::charWidth({:?})", char);

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "charWidth", "(C)I", (char,)).await
    }

    async fn chars_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::charsWidth({:?}, {:?}, {:?})", chars, offset, len);

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "charsWidth", "([CII)I", (chars, offset, len)).await
    }

    pub async fn midp_font(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<MidpFont>> {
        jvm.get_field(this, "midpFont", "Ljavax/microedition/lcdui/Font;").await
    }
}
