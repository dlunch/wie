use alloc::string::String as RustString;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::canvas::string_width;
use wie_jvm_support::WieJvmContext;

// class javax.microedition.lcdui.Font
#[java_class(name = "javax/microedition/lcdui/Font", public, final)]
pub struct Font {
    #[field(public, static, final)]
    FACE_SYSTEM: i32,
    #[field(public, static, final)]
    FACE_MONOSPACE: i32,
    #[field(public, static, final)]
    FACE_PROPORTIONAL: i32,
    #[field(public, static, final)]
    STYLE_PLAIN: i32,
    #[field(public, static, final)]
    STYLE_BOLD: i32,
    #[field(public, static, final)]
    STYLE_ITALIC: i32,
    #[field(public, static, final)]
    STYLE_UNDERLINED: i32,
    #[field(public, static, final)]
    SIZE_SMALL: i32,
    #[field(public, static, final)]
    SIZE_MEDIUM: i32,
    #[field(public, static, final)]
    SIZE_LARGE: i32,
}

#[java_methods]
impl Font {
    pub const HEIGHT: i32 = 12;

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Font::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I", 32).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_PROPORTIONAL", "I", 64)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_BOLD", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_ITALIC", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_UNDERLINED", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_MEDIUM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I", 8).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_LARGE", "I", 16).await?;

        Ok(())
    }

    #[method(constructor)]
    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Font::<init>({this:?})");

        Ok(())
    }

    #[method(descriptor = "()I", public)]
    async fn get_height(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getHeight");

        Ok(Self::HEIGHT) // TODO: hardcoded
    }

    #[method(public)]
    async fn get_baseline_position(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getBaselinePosition({this:?})");

        Ok(10)
    }

    #[method(public, static)]
    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    #[method(public, static)]
    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getFont({face:?}, {style:?}, {size:?})");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    #[method(name = "getFont", public, static)]
    async fn get_font_by_specifier(jvm: &Jvm, _: &mut WieJvmContext, font_specifier: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getFont({font_specifier})");

        jvm.invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await
    }

    #[method(public)]
    async fn get_face(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getFace({this:?})");

        Ok(0) // FACE_SYSTEM
    }

    #[method(public)]
    async fn get_style(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getStyle({this:?})");

        Ok(0) // STYLE_PLAIN
    }

    #[method(public)]
    async fn get_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getSize({this:?})");

        Ok(0) // SIZE_MEDIUM
    }

    #[method(public)]
    async fn is_plain(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Font::isPlain({this:?})");

        Ok(true)
    }

    #[method(public)]
    async fn is_bold(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Font::isBold({this:?})");

        Ok(false)
    }

    #[method(public)]
    async fn is_italic(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Font::isItalic({this:?})");

        Ok(false)
    }

    #[method(public)]
    async fn is_underlined(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Font::isUnderlined({this:?})");

        Ok(false)
    }

    #[method(public)]
    async fn string_width(jvm: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::stringWidth({string:?})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(string_width(context.system().platform().font(), &string, 10.0) as _)
    }

    #[method(public)]
    async fn substring_width(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        _: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::substringWidth({string:?}, {offset:?}, {len:?})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        Ok(string_width(context.system().platform().font(), &substring, 10.0) as _)
    }

    #[method(public)]
    async fn char_width(_: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::charWidth({char:?})");

        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(string_width(context.system().platform().font(), &string, 10.0) as _)
    }

    #[method(public)]
    async fn chars_width(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        _: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::charsWidth({chars:?}, {offset:?}, {len:?})");

        let chars = jvm.load_array(&chars, offset as _, len as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        Ok(string_width(context.system().platform().font(), &string, 10.0) as _)
    }
}
