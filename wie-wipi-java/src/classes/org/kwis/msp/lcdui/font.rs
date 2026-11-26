use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::Font as MidpFont;

// class org.kwis.msp.lcdui.Font
#[java_class(name = "org/kwis/msp/lcdui/Font", public)]
pub struct Font {
    #[field(private)]
    midpFont: ClassInstanceRef<MidpFont>,
    #[field(protected)]
    face: i32,
    #[field(protected)]
    style: i32,
    #[field(protected)]
    size: i32,
    #[field(public, static)]
    FACE_SYSTEM: i32,
    #[field(public, static)]
    FACE_MONOSPACE: i32,
    #[field(public, static)]
    FACE_PROPORTIONAL: i32,
    #[field(public, static)]
    STYLE_PLAIN: i32,
    #[field(public, static)]
    STYLE_BOLD: i32,
    #[field(public, static)]
    STYLE_ITALIC: i32,
    #[field(public, static)]
    STYLE_UNDERLINED: i32,
    #[field(public, static)]
    SIZE_SMALL: i32,
    #[field(public, static)]
    SIZE_MEDIUM: i32,
    #[field(public, static)]
    SIZE_LARGE: i32,
}

#[java_methods]
impl Font {
    #[method(static_initializer)]
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

    #[method(constructor, private)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, midp_font: ClassInstanceRef<MidpFont>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Font::<init>({this:?})");

        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/Font",
            "midpFont",
            "Ljavax/microedition/lcdui/Font;",
            midp_font,
        )
        .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Font", "face", "I", 0).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Font", "style", "I", 0).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Font", "size", "I", 0).await?;

        Ok(())
    }

    #[method(public)]
    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getHeight");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        jvm.invoke_virtual(&midp_font, "javax/microedition/lcdui/Font", "getHeight", "()I", ())
            .await
    }

    #[method(public)]
    async fn get_baseline_position(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Font::getBaselinePosition({this:?})");

        Ok(0)
    }

    #[method(public)]
    async fn get_face(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getFace({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "face", "I").await
    }

    #[method(public)]
    async fn get_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getSize({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "size", "I").await
    }

    #[method(public)]
    async fn get_style(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::getStyle({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "style", "I").await
    }

    #[method(public)]
    async fn is_bold(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isBold({this:?})");

        let style: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "style", "I").await?;
        Ok(style & 1 != 0)
    }

    #[method(public)]
    async fn is_italic(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isItalic({this:?})");

        let style: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "style", "I").await?;
        Ok(style & 2 != 0)
    }

    #[method(public)]
    async fn is_plain(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isPlain({this:?})");

        let style: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "style", "I").await?;
        Ok(style == 0)
    }

    #[method(public)]
    async fn is_underlined(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Font::isUnderlined({this:?})");

        let style: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Font", "style", "I").await?;
        Ok(style & 4 != 0)
    }

    #[method(public, static)]
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

    #[method(public, static)]
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
        jvm.put_field(&mut instance, "org/kwis/msp/lcdui/Font", "face", "I", face).await?;
        jvm.put_field(&mut instance, "org/kwis/msp/lcdui/Font", "style", "I", style).await?;
        jvm.put_field(&mut instance, "org/kwis/msp/lcdui/Font", "size", "I", size).await?;

        Ok(instance)
    }

    #[method(public)]
    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::stringWidth({string:?})");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        jvm.invoke_virtual(
            &midp_font,
            "javax/microedition/lcdui/Font",
            "stringWidth",
            "(Ljava/lang/String;)I",
            (string,),
        )
        .await
    }

    #[method(public)]
    async fn substring_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::substringWidth({string:?}, {offset:?}, {len:?})");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        jvm.invoke_virtual(
            &midp_font,
            "javax/microedition/lcdui/Font",
            "substringWidth",
            "(Ljava/lang/String;II)I",
            (string, offset, len),
        )
        .await
    }

    #[method(public)]
    async fn char_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::charWidth({char:?})");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        jvm.invoke_virtual(&midp_font, "javax/microedition/lcdui/Font", "charWidth", "(C)I", (char,))
            .await
    }

    #[method(public)]
    async fn chars_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Font::charsWidth({chars:?}, {offset:?}, {len:?})");

        let midp_font: ClassInstanceRef<MidpFont> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await?;
        jvm.invoke_virtual(&midp_font, "javax/microedition/lcdui/Font", "charsWidth", "([CII)I", (chars, offset, len))
            .await
    }

    pub async fn midp_font(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<MidpFont>> {
        jvm.get_field(this, "org/kwis/msp/lcdui/Font", "midpFont", "Ljavax/microedition/lcdui/Font;")
            .await
    }
}
