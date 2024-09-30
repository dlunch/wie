use alloc::{string::String as RustString, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};

use wie_backend::canvas;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Font
pub struct Font;

impl Font {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, Default::default()),
                JavaMethodProto::new("substringWidth", "(Ljava/lang/String;II)I", Self::substring_width, Default::default()),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, Default::default()),
                JavaMethodProto::new(
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    Self::get_font,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_MONOSPACE", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.msp.lcdui.Font::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I", 32).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I", 8).await?;

        Ok(())
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::<init>({:?})", &this);

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::getHeight");

        Ok(12) // TODO: hardcoded
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::getFont({:?}, {:?}, {:?})", face, style, size);

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::stringWidth({:?})", &string);

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(canvas::string_width(&string, 10.0) as _)
    }

    async fn substring_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        _: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!(
            "stub javax.microedition.msp.lcdui.Font::substringWidth({:?}, {:?}, {:?})",
            &string,
            offset,
            len
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        Ok(canvas::string_width(&substring, 10.0) as _)
    }

    async fn char_width(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.msp.lcdui.Font::charWidth({:?})", char);

        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(canvas::string_width(&string, 10.0) as _)
    }
}
