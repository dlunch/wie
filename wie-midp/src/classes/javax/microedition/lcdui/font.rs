use alloc::{
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::{Font as BackendFont, canvas};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Font
pub struct Font;

impl Font {
    pub const HEIGHT: i32 = 12;

    pub fn text_width(font: &BackendFont, text: &str) -> i32 {
        canvas::string_width(font, text, 10.0).ceil() as i32
    }

    pub fn minimum_width(font: &BackendFont, text: &str) -> i32 {
        text.chars()
            .filter(|character| *character != '\n')
            .map(|character| Self::text_width(font, &character.to_string()))
            .max()
            .unwrap_or(0)
    }

    pub fn preferred_width(font: &BackendFont, text: &str) -> i32 {
        text.split('\n').map(|line| Self::text_width(font, line)).max().unwrap_or(0)
    }

    pub fn wrap(font: &BackendFont, text: &str, maximum_width: Option<i32>) -> Vec<RustString> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let characters = paragraph.chars().collect::<Vec<_>>();
            if characters.is_empty() {
                lines.push(RustString::new());
                continue;
            }

            let Some(maximum_width) = maximum_width else {
                lines.push(paragraph.to_string());
                continue;
            };
            let maximum_width = maximum_width.max(1);
            let mut start = 0;
            while start < characters.len() {
                let mut end = start;
                let mut width = 0;
                let mut word_boundary = None;
                while end < characters.len() {
                    let character_width = Self::text_width(font, &characters[end].to_string());
                    if end > start && width + character_width > maximum_width {
                        break;
                    }
                    width += character_width;
                    end += 1;
                    if characters[end - 1].is_whitespace() {
                        word_boundary = Some(end);
                    }
                }

                let split = if end == characters.len() {
                    end
                } else {
                    word_boundary.filter(|boundary| *boundary > start).unwrap_or(end.max(start + 1))
                };
                let line = characters[start..split].iter().collect::<RustString>();
                lines.push(line.trim_end().to_string());
                start = split;
                while start < characters.len() && characters[start].is_whitespace() {
                    start += 1;
                }
            }
        }

        lines
    }

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "substringWidth",
                    "(Ljava/lang/String;II)I",
                    Self::substring_width,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("charsWidth", "([CII)I", Self::chars_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    Self::get_font,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "FACE_SYSTEM",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "FACE_MONOSPACE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "FACE_PROPORTIONAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_PLAIN",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_BOLD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_ITALIC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STYLE_UNDERLINED",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_SMALL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_MEDIUM",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIZE_LARGE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

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

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Font::<init>({this:?})");

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getHeight");

        Ok(Self::HEIGHT) // TODO: hardcoded
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getFont({face:?}, {style:?}, {size:?})");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn string_width(jvm: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::stringWidth({string:?})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(canvas::string_width(context.system().platform().font(), &string, 10.0) as _)
    }

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

        Ok(canvas::string_width(context.system().platform().font(), &substring, 10.0) as _)
    }

    async fn char_width(_: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::charWidth({char:?})");

        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(canvas::string_width(context.system().platform().font(), &string, 10.0) as _)
    }

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

        Ok(canvas::string_width(context.system().platform().font(), &string, 10.0) as _)
    }
}

#[cfg(test)]
mod test {
    use test_utils::TestPlatform;
    use wie_backend::Platform;

    use super::*;

    #[test]
    fn preserves_explicit_newlines_and_prefers_word_boundaries() {
        let platform = TestPlatform::new();
        let width = Font::text_width(platform.font(), "alpha ");
        assert_eq!(
            Font::wrap(platform.font(), "alpha beta\n\ngamma", Some(width)),
            ["alpha", "beta", "", "gamma"]
        );
    }

    #[test]
    fn falls_back_to_character_boundaries_for_long_words() {
        let platform = TestPlatform::new();
        let width = Font::text_width(platform.font(), "ab");
        assert_eq!(Font::wrap(platform.font(), "abcdef", Some(width)), ["ab", "cd", "ef"]);
    }
}
