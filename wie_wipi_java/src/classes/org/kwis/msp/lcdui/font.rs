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
                JavaMethodProto::new("getBaselinePosition", "()I", Self::get_baseline_position, Default::default()),
                JavaMethodProto::new("getFace", "()I", Self::get_face, Default::default()),
                JavaMethodProto::new("getSize", "()I", Self::get_size, Default::default()),
                JavaMethodProto::new("getStyle", "()I", Self::get_style, Default::default()),
                JavaMethodProto::new("isBold", "()Z", Self::is_bold, Default::default()),
                JavaMethodProto::new("isItalic", "()Z", Self::is_italic, Default::default()),
                JavaMethodProto::new("isPlain", "()Z", Self::is_plain, Default::default()),
                JavaMethodProto::new("isUnderlined", "()Z", Self::is_underlined, Default::default()),
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
                JavaFieldProto::new("face", "I", Default::default()),
                JavaFieldProto::new("style", "I", Default::default()),
                JavaFieldProto::new("size", "I", Default::default()),
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_MONOSPACE", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_PROPORTIONAL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_BOLD", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_ITALIC", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_UNDERLINED", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_MEDIUM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_LARGE", "I", FieldAccessFlags::STATIC),
            ],
            access_flags: Default::default(),
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<clinit>");

        let face_system: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I").await?;
        let face_monospace: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I").await?;
        let face_proportional: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "FACE_PROPORTIONAL", "I").await?;
        let style_plain: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I").await?;
        let style_bold: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "STYLE_BOLD", "I").await?;
        let style_italic: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "STYLE_ITALIC", "I").await?;
        let style_underlined: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "STYLE_UNDERLINED", "I").await?;
        let size_small: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I").await?;
        let size_medium: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "SIZE_MEDIUM", "I").await?;
        let size_large: i32 = jvm.get_static_field("javax/microedition/lcdui/Font", "SIZE_LARGE", "I").await?;

        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_SYSTEM", "I", face_system).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_MONOSPACE", "I", face_monospace)
            .await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "FACE_PROPORTIONAL", "I", face_proportional)
            .await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_PLAIN", "I", style_plain).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_BOLD", "I", style_bold).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_ITALIC", "I", style_italic).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "STYLE_UNDERLINED", "I", style_underlined)
            .await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_SMALL", "I", size_small).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_MEDIUM", "I", size_medium).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Font", "SIZE_LARGE", "I", size_large).await?;

        Ok(())
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, midp_font: ClassInstanceRef<MidpFont>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<init>({this:?})");

        jvm.put_field(&mut this, "midpFont", "Ljavax/microedition/lcdui/Font;", midp_font).await?;
        jvm.put_field(&mut this, "face", "I", 0).await?;
        jvm.put_field(&mut this, "style", "I", 0).await?;
        jvm.put_field(&mut this, "size", "I", 0).await?;

        Ok(())
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getHeight");

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "getHeight", "()I", ()).await
    }

    async fn get_baseline_position(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getBaselinePosition({this:?})");

        Ok(0)
    }

    async fn get_face(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getFace({this:?})");

        jvm.get_field(&this, "face", "I").await
    }

    async fn get_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getSize({this:?})");

        jvm.get_field(&this, "size", "I").await
    }

    async fn get_style(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getStyle({this:?})");

        jvm.get_field(&this, "style", "I").await
    }

    async fn is_bold(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isBold({this:?})");

        let style: i32 = jvm.get_field(&this, "style", "I").await?;
        Ok(style & 1 != 0)
    }

    async fn is_italic(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isItalic({this:?})");

        let style: i32 = jvm.get_field(&this, "style", "I").await?;
        Ok(style & 2 != 0)
    }

    async fn is_plain(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isPlain({this:?})");

        let style: i32 = jvm.get_field(&this, "style", "I").await?;
        Ok(style == 0)
    }

    async fn is_underlined(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isUnderlined({this:?})");

        let style: i32 = jvm.get_field(&this, "style", "I").await?;
        Ok(style & 4 != 0)
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
        tracing::debug!("org.kwis.msp.lcdui.Font::getFont({face:?}, {style:?}, {size:?})");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Font",
                "getFont",
                "(III)Ljavax/microedition/lcdui/Font;",
                (face, style, size),
            )
            .await?;

        let mut instance: ClassInstanceRef<Font> = jvm
            .new_class("org/kwis/msp/lcdui/Font", "(Ljavax/microedition/lcdui/Font;)V", (midp_font,))
            .await?
            .into();
        jvm.put_field(&mut instance, "face", "I", face).await?;
        jvm.put_field(&mut instance, "style", "I", style).await?;
        jvm.put_field(&mut instance, "size", "I", size).await?;

        Ok(instance)
    }

    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::stringWidth({string:?})");

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
        tracing::debug!("org.kwis.msp.lcdui.Font::substringWidth({string:?}, {offset:?}, {len:?})");

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "substringWidth", "(Ljava/lang/String;II)I", (string, offset, len))
            .await
    }

    async fn char_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::charWidth({char:?})");

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
        tracing::debug!("org.kwis.msp.lcdui.Font::charsWidth({chars:?}, {offset:?}, {len:?})");

        let midp_font = jvm.get_field(&this, "midpFont", "Ljavax/microedition/lcdui/Font;").await?;
        jvm.invoke_virtual(&midp_font, "charsWidth", "([CII)I", (chars, offset, len)).await
    }

    pub async fn midp_font(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<MidpFont>> {
        jvm.get_field(this, "midpFont", "Ljavax/microedition/lcdui/Font;").await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::lcdui::Font, get_protos};

    #[test]
    fn test_font_attributes_round_trip() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let font: ClassInstanceRef<Font> = jvm
                .invoke_static(
                    "org/kwis/msp/lcdui/Font",
                    "getFont",
                    "(III)Lorg/kwis/msp/lcdui/Font;",
                    (32, 1 | 2 | 4, 16),
                )
                .await?;

            assert_eq!(jvm.invoke_virtual::<_, i32>(&font, "getFace", "()I", ()).await?, 32);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&font, "getStyle", "()I", ()).await?, 7);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&font, "getSize", "()I", ()).await?, 16);
            assert!(jvm.invoke_virtual::<_, bool>(&font, "isBold", "()Z", ()).await?);
            assert!(jvm.invoke_virtual::<_, bool>(&font, "isItalic", "()Z", ()).await?);
            assert!(jvm.invoke_virtual::<_, bool>(&font, "isUnderlined", "()Z", ()).await?);
            assert!(!jvm.invoke_virtual::<_, bool>(&font, "isPlain", "()Z", ()).await?);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&font, "getBaselinePosition", "()I", ()).await?, 0);

            Ok(())
        })
    }
}
