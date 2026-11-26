use alloc::{string::String as RustString, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::text_layout::wrap;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Font, Graphics, Item},
    net::wie::MIDPKeyCode,
};

const INSET: i32 = 2;
const LEFT_TOP: i32 = 4 | 16;
const ANY: i32 = 0;
const EMAILADDR: i32 = 1;
const NUMERIC: i32 = 2;
const PHONENUMBER: i32 = 3;
const URL: i32 = 4;
const DECIMAL: i32 = 5;
const PASSWORD: i32 = 0x10000;
const UNEDITABLE: i32 = 0x20000;
const SENSITIVE: i32 = 0x40000;
const NON_PREDICTIVE: i32 = 0x80000;
const INITIAL_CAPS_WORD: i32 = 0x100000;
const INITIAL_CAPS_SENTENCE: i32 = 0x200000;
const CONSTRAINT_MASK: i32 = 0xffff;

// class javax.microedition.lcdui.TextField
pub struct TextField;

impl TextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextField",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getChars", "([C)I", Self::get_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setChars", "([CII)V", Self::set_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("insert", "(Ljava/lang/String;I)V", Self::insert_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("insert", "([CIII)V", Self::insert_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("delete", "(II)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxSize", "()I", Self::get_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxSize", "(I)I", Self::set_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCaretPosition", "()I", Self::get_caret_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getConstraints", "()I", Self::get_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setConstraints", "(I)V", Self::set_constraints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setInitialInputMode",
                    "(Ljava/lang/String;)V",
                    Self::set_initial_input_mode,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("minimumContentWidth", "()I", Self::minimum_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new("minimumContentHeight", "()I", Self::minimum_content_height, MethodAccessFlags::empty()),
                JavaMethodProto::new("preferredContentWidth", "()I", Self::preferred_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "preferredContentHeight",
                    "(I)I",
                    Self::preferred_content_height,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "paintContent",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    Self::paint_content,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("isFocusable", "()Z", Self::is_focusable, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleItemKey", "(I)I", Self::handle_item_key, MethodAccessFlags::empty()),
            ],
            fields: vec![
                JavaFieldProto::new("ANY", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "EMAILADDR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NUMERIC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PHONENUMBER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("URL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "DECIMAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PASSWORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "UNEDITABLE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SENSITIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NON_PREDICTIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_WORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_SENTENCE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "CONSTRAINT_MASK",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/TextField", "ANY", "I", ANY).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "EMAILADDR", "I", EMAILADDR)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "NUMERIC", "I", NUMERIC)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "PHONENUMBER", "I", PHONENUMBER)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "URL", "I", URL).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "DECIMAL", "I", DECIMAL)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "PASSWORD", "I", PASSWORD)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "UNEDITABLE", "I", UNEDITABLE)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "SENSITIVE", "I", SENSITIVE)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "NON_PREDICTIVE", "I", NON_PREDICTIVE)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "INITIAL_CAPS_WORD", "I", INITIAL_CAPS_WORD)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "INITIAL_CAPS_SENTENCE", "I", INITIAL_CAPS_SENTENCE)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "CONSTRAINT_MASK", "I", CONSTRAINT_MASK)
            .await?;

        Ok(())
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<init>({this:?}, {label:?}, {text:?}, {max_size}, {constraints})");
        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Non-positive text capacity").await);
        }
        Self::check_constraints(jvm, constraints).await?;
        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Item", "setLabel", "(Ljava/lang/String;)V", (label,))
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextField", "maxSize", "I", max_size)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextField", "constraints", "I", constraints)
            .await?;
        jvm.invoke_special(&this, "javax/microedition/lcdui/TextField", "setString", "(Ljava/lang/String;)V", (text,))
            .await
    }

    async fn check_constraints(jvm: &Jvm, constraints: i32) -> JvmResult<()> {
        let allowed = CONSTRAINT_MASK | PASSWORD | UNEDITABLE | SENSITIVE | NON_PREDICTIVE | INITIAL_CAPS_WORD | INITIAL_CAPS_SENTENCE;
        if constraints & !allowed != 0 || (constraints & CONSTRAINT_MASK) > DECIMAL {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid text constraints").await);
        }
        Ok(())
    }

    fn accepts(text: &[JavaChar], constraints: i32) -> bool {
        if text.is_empty() {
            return true;
        }
        match constraints & CONSTRAINT_MASK {
            NUMERIC => {
                let digits = text.strip_prefix(&[b'-' as JavaChar]).unwrap_or(text);
                !digits.is_empty() && digits.iter().all(|&c| matches!(u8::try_from(c), Ok(b'0'..=b'9')))
            }
            DECIMAL => {
                let text = RustString::from_utf16_lossy(text);
                let unsigned = text.strip_prefix('-').unwrap_or(&text);
                let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
                !(whole.is_empty() && fraction.is_empty()) && whole.bytes().chain(fraction.bytes()).all(|b| b.is_ascii_digit())
            }
            PHONENUMBER => text.iter().all(|&c| matches!(u8::try_from(c), Ok(b'0'..=b'9' | b'+' | b'*' | b'#'))),
            _ => true,
        }
    }

    async fn contents(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<JavaChar>> {
        let text: ClassInstanceRef<String> = jvm
            .get_field(this, "javax/microedition/lcdui/TextField", "text", "Ljava/lang/String;")
            .await?;
        JavaLangString::to_utf16(jvm, &text).await
    }

    async fn display_text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<RustString> {
        let text = Self::contents(jvm, this).await?;
        let constraints: i32 = jvm.get_field(this, "javax/microedition/lcdui/TextField", "constraints", "I").await?;
        Ok(if constraints & PASSWORD != 0 {
            "*".repeat(text.len())
        } else {
            RustString::from_utf16_lossy(&text)
        })
    }

    async fn replace(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: Vec<JavaChar>) -> JvmResult<()> {
        let max_size: i32 = jvm.get_field(this, "javax/microedition/lcdui/TextField", "maxSize", "I").await?;
        let constraints: i32 = jvm.get_field(this, "javax/microedition/lcdui/TextField", "constraints", "I").await?;
        if text.len() > max_size as usize || !Self::accepts(&text, constraints) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Text exceeds capacity or violates constraints")
                .await);
        }
        Self::store_text(jvm, this, text).await
    }

    async fn store_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: Vec<JavaChar>) -> JvmResult<()> {
        let text = JavaLangString::from_utf16(jvm, text).await?;
        jvm.put_field(this, "javax/microedition/lcdui/TextField", "text", "Ljava/lang/String;", text)
            .await?;
        jvm.invoke_virtual(this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn insert_units(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, position: i32, src: Vec<JavaChar>) -> JvmResult<()> {
        let mut text = Self::contents(jvm, this).await?;
        let position = position.clamp(0, text.len() as i32) as usize;
        text.splice(position..position, src);
        Self::replace(jvm, this, text).await
    }

    async fn array_slice(jvm: &Jvm, data: &ClassInstanceRef<Array<JavaChar>>, offset: i32, length: i32) -> JvmResult<Vec<JavaChar>> {
        if offset < 0 || length < 0 {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "Negative character range")
                .await);
        }
        jvm.load_array(data, offset as usize, length as usize).await
    }

    async fn get_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.TextField::getString({this:?})");
        jvm.get_field(&this, "javax/microedition/lcdui/TextField", "text", "Ljava/lang/String;")
            .await
    }

    async fn set_string(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setString({this:?}, {text:?})");
        let text = if text.is_null() {
            Vec::new()
        } else {
            JavaLangString::to_utf16(jvm, &text).await?
        };
        Self::replace(jvm, &mut this, text).await
    }

    async fn get_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut data: ClassInstanceRef<Array<JavaChar>>,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getChars({this:?}, {data:?})");
        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Null destination").await);
        }
        let text = Self::contents(jvm, &this).await?;
        let size = text.len();
        jvm.store_array(&mut data, 0, text).await?;
        Ok(size as i32)
    }

    async fn set_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setChars({this:?}, {data:?}, {offset}, {length})");
        let text = if data.is_null() {
            Vec::new()
        } else {
            Self::array_slice(jvm, &data, offset, length).await?
        };
        Self::replace(jvm, &mut this, text).await
    }

    async fn insert_string(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::insert({this:?}, {text:?}, {position})");
        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Null insertion").await);
        }
        let src = JavaLangString::to_utf16(jvm, &text).await?;
        Self::insert_units(jvm, &mut this, position, src).await
    }

    async fn insert_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::insert({this:?}, {data:?}, {offset}, {length}, {position})");
        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Null insertion").await);
        }
        let src = Self::array_slice(jvm, &data, offset, length).await?;
        Self::insert_units(jvm, &mut this, position, src).await
    }

    async fn delete(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, offset: i32, length: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::delete({this:?}, {offset}, {length})");
        let mut text = Self::contents(jvm, &this).await?;
        if offset < 0 || length < 0 || offset > text.len() as i32 - length {
            return Err(jvm.exception("java/lang/StringIndexOutOfBoundsException", "Invalid deletion range").await);
        }
        text.drain(offset as usize..(offset + length) as usize);
        Self::replace(jvm, &mut this, text).await
    }

    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getMaxSize({this:?})");
        jvm.get_field(&this, "javax/microedition/lcdui/TextField", "maxSize", "I").await
    }

    async fn set_max_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::setMaxSize({this:?}, {max_size})");
        if max_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Non-positive text capacity").await);
        }
        let mut text = Self::contents(jvm, &this).await?;
        if text.len() > max_size as usize {
            text.truncate(max_size as usize);
            Self::replace(jvm, &mut this, text).await?;
        }
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextField", "maxSize", "I", max_size)
            .await?;
        Ok(max_size)
    }

    async fn size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::size({this:?})");
        let text: ClassInstanceRef<String> = jvm
            .get_field(&this, "javax/microedition/lcdui/TextField", "text", "Ljava/lang/String;")
            .await?;
        jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await
    }

    async fn get_caret_position(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getCaretPosition({this:?})");
        jvm.invoke_special(&this, "javax/microedition/lcdui/TextField", "size", "()I", ()).await
    }

    async fn get_constraints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::getConstraints({this:?})");
        jvm.get_field(&this, "javax/microedition/lcdui/TextField", "constraints", "I").await
    }

    async fn set_constraints(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setConstraints({this:?}, {constraints})");
        Self::check_constraints(jvm, constraints).await?;
        let text = Self::contents(jvm, &this).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextField", "constraints", "I", constraints)
            .await?;
        if !Self::accepts(&text, constraints) {
            return Self::store_text(jvm, &mut this, Vec::new()).await;
        }
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn set_initial_input_mode(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        subset: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::setInitialInputMode({this:?}, {subset:?})");
        // The host selects the input mode for its native editor.
        Ok(())
    }

    async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::minimumContentWidth({this:?})");
        Ok(24)
    }

    async fn minimum_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::minimumContentHeight({this:?})");
        Ok(Font::HEIGHT + INSET * 2)
    }

    async fn preferred_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::preferredContentWidth({this:?})");
        Ok(96)
    }

    async fn preferred_content_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::preferredContentHeight({this:?}, {width})");
        let text = Self::display_text(jvm, &this).await?;
        let width = (width >= 0).then(|| (width - INSET * 2).max(1));
        Ok(wrap(context.system().platform().font(), &text, 10.0, width).len() as i32 * Font::HEIGHT + INSET * 2)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        focused: bool,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::paintContent({this:?}, {graphics:?}, {x}, {y}, {width}, {height}, {focused})");
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x596773,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawRect",
                "(IIII)V",
                (x, y, width - 1, height - 1),
            )
            .await?;
        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setFont",
                "(Ljavax/microedition/lcdui/Font;)V",
                (font,),
            )
            .await?;
        let text = Self::display_text(jvm, &this).await?;
        let clip_y: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
            .await?;
        let clip_height: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
            .await?;
        for (index, line) in wrap(context.system().platform().font(), &text, 10.0, Some((width - INSET * 2).max(1)))
            .iter()
            .enumerate()
        {
            let line_y = y + INSET + index as i32 * Font::HEIGHT;
            if line_y + Font::HEIGHT <= clip_y {
                continue;
            }
            if line_y >= (y + height).min(clip_y + clip_height) {
                break;
            }
            let line = JavaLangString::from_rust_string(jvm, line).await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (line, x + INSET, line_y, LEFT_TOP),
                )
                .await?;
        }
        Ok(())
    }

    async fn is_focusable(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.TextField::isFocusable({this:?})");
        Ok(true)
    }

    async fn handle_item_key(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextField::handleItemKey({this:?}, {key})");
        if key != MIDPKeyCode::FIRE as i32 {
            return Ok(0);
        }
        let constraints: i32 = jvm.get_field(&this, "javax/microedition/lcdui/TextField", "constraints", "I").await?;
        if constraints & UNEDITABLE != 0 {
            return Ok(0);
        }
        let current = Self::contents(jvm, &this).await?;
        let Some(result) = context.system().platform().text_input(&RustString::from_utf16_lossy(&current)).await else {
            return Ok(Item::INPUT_HANDLED);
        };
        // A handset stops accepting characters at the capacity, so a longer host entry keeps its head.
        let max_size: i32 = jvm.get_field(&this, "javax/microedition/lcdui/TextField", "maxSize", "I").await?;
        let mut text: Vec<_> = result.encode_utf16().collect();
        text.truncate(max_size as usize);
        if text == current || !Self::accepts(&text, constraints) {
            return Ok(Item::INPUT_HANDLED);
        }
        Self::store_text(jvm, &mut this, text).await?;
        Ok(Item::INPUT_HANDLED | Item::INPUT_CHANGED)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use super::{ANY, DECIMAL, NUMERIC};
    use crate::get_protos;

    #[test]
    fn text_changes_enforce_capacity_and_constraints_before_replacing_contents() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let initial = JavaLangString::from_rust_string(&jvm, "-.5").await?;
            let field = jvm
                .new_class(
                    "javax/microedition/lcdui/TextField",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    (None, initial, 4, DECIMAL),
                )
                .await?;
            for text in ["12.34", "1e2", "+1"] {
                let text = JavaLangString::from_rust_string(&jvm, text).await?;
                let result: JvmResult<()> = jvm
                    .invoke_virtual(
                        &field,
                        "javax/microedition/lcdui/TextField",
                        "setString",
                        "(Ljava/lang/String;)V",
                        (text,),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("invalid replacement accepted")
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
            }
            let result: JvmResult<i32> = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setMaxSize", "(I)I", (2,))
                .await;
            let Err(JavaError::JavaException(exception)) = result else {
                panic!("invalid truncated decimal accepted")
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&field, "javax/microedition/lcdui/TextField", "getMaxSize", "()I", ())
                    .await?,
                4
            );
            let value: ClassInstanceRef<String> = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &value).await?, "-.5");

            let _: () = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setConstraints", "(I)V", (NUMERIC,))
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&field, "javax/microedition/lcdui/TextField", "size", "()I", ())
                    .await?,
                0
            );
            let _: () = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setConstraints", "(I)V", (ANY,))
                .await?;
            let text = JavaLangString::from_rust_string(&jvm, "📝").await?;
            let _: () = jvm
                .invoke_virtual(
                    &field,
                    "javax/microedition/lcdui/TextField",
                    "insert",
                    "(Ljava/lang/String;I)V",
                    (text, 10),
                )
                .await?;
            let _: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setMaxSize", "(I)I", (1,))
                .await?;
            let chars = jvm.instantiate_array("C", 2).await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&field, "javax/microedition/lcdui/TextField", "getChars", "([C)I", (chars.clone(),))
                    .await?,
                1
            );
            assert_eq!(jvm.load_array::<u16>(&chars, 0, 2).await?, [0xd83d, 0]);
            Ok(())
        })
    }
}
