use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::net::wie::KeyboardEventType;

use crate::classes::{
    net::wie::WIPIKeyCode,
    org::kwis::msp::lcdui::{Font, Graphics},
};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init_with_text, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("insert", "([CIII)V", Self::insert, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("delete", "(II)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxLength", "()I", Self::get_max_length, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getConstraint", "()I", Self::get_constraint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paintContent",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_content,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setFont", "(Lorg/kwis/msp/lcdui/Font;)V", Self::set_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFont", "()Lorg/kwis/msp/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("calcPreferredSize", "(I)V", Self::calc_preferred_size, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("charCount", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("constraint", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("f", "Lorg/kwis/msp/lcdui/Font;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("m_cPos", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("m_td", "[C", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("maxLength", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("imHandler", "Lorg/kwis/msp/lcdui/InputMethodHandler;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("iMode", "I", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::<init>({this:?})");
        jvm.invoke_special(
            &this,
            "org/kwis/msp/lwc/TextComponent",
            "<init>",
            "(Ljava/lang/String;I)V",
            (ClassInstanceRef::<String>::new(None), 0),
        )
        .await
    }

    async fn init_with_text(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::<init>({this:?}, {data:?}, {constraint})");
        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        if !(0..=2).contains(&constraint) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Unsupported text constraint").await);
        }
        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "mask", "I", mask | 4).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "bg", "I", 0xffffff).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "maxLength", "I", -1).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "constraint", "I", constraint)
            .await?;
        let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (constraint,)).await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/TextComponent",
            "imHandler",
            "Lorg/kwis/msp/lcdui/InputMethodHandler;",
            handler,
        )
        .await?;
        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("org/kwis/msp/lcdui/Font", "getDefaultFont", "()Lorg/kwis/msp/lcdui/Font;", ())
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "f", "Lorg/kwis/msp/lcdui/Font;", font)
            .await?;
        jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (data,))
            .await
    }

    async fn get_constraint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getConstraint({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "constraint", "I").await
    }

    async fn get_max_length(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getMaxLength({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "maxLength", "I").await
    }

    async fn set_max_length(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setMaxLength({this:?}, {max})");
        if max == 0 || max < -1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid maximum length").await);
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "maxLength", "I", max).await?;
        let count: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "charCount", "I").await?;
        if max > 0 && count > max {
            let _: () = jvm
                .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "delete", "(II)V", (max, count - max))
                .await?;
        }
        Ok(())
    }

    fn accepts(constraint: i32, chars: &[u16]) -> bool {
        constraint == 0 || chars.iter().all(|c| matches!(c, 48..=57) || constraint == 1 && matches!(c, 32 | 45))
    }

    async fn chars<T: Sync>(jvm: &Jvm, this: &ClassInstanceRef<T>) -> JvmResult<Vec<u16>> {
        let data: ClassInstanceRef<Array<u16>> = jvm.get_field(this, "org/kwis/msp/lwc/TextComponent", "m_td", "[C").await?;
        let count: i32 = jvm.get_field(this, "org/kwis/msp/lwc/TextComponent", "charCount", "I").await?;
        jvm.load_array(&data, 0, count as usize).await
    }

    async fn store(jvm: &Jvm, mut this: ClassInstanceRef<Self>, chars: Vec<u16>, cursor: i32) -> JvmResult<()> {
        let count = chars.len();
        let mut data = jvm.instantiate_array("C", count).await?;
        jvm.store_array(&mut data, 0, chars).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "m_td", "[C", data).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "charCount", "I", count as i32)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "m_cPos", "I", cursor).await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, data: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setString({this:?}, {data:?})");
        let mut chars = if data.is_null() {
            Vec::new()
        } else {
            JavaLangString::to_utf16(jvm, &data).await?
        };
        let constraint = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "constraint", "I").await?;
        if !Self::accepts(constraint, &chars) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Text violates input constraint")
                .await);
        }
        let max: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "maxLength", "I").await?;
        if max > 0 {
            chars.truncate(max as usize);
        }
        let cursor = chars.len() as i32;
        Self::store(jvm, this, chars, cursor).await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getString({this:?})");
        Ok(JavaLangString::from_utf16(jvm, Self::chars(jvm, &this).await?).await?.into())
    }

    async fn insert(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<u16>>,
        offset: i32,
        len: i32,
        index: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::insert({this:?}, {data:?}, {offset}, {len}, {index})");
        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }
        let mut chars = Self::chars(jvm, &this).await?;
        if index < 0 || index as usize > chars.len() || offset < 0 || len < 0 || offset as usize + len as usize > jvm.array_length(&data).await? {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid insertion range").await);
        }
        let max: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "maxLength", "I").await?;
        if max > 0 && chars.len() + len as usize > max as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Text exceeds maximum length").await);
        }
        let inserted = jvm.load_array(&data, offset as usize, len as usize).await?;
        let constraint = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "constraint", "I").await?;
        if !Self::accepts(constraint, &inserted) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Text violates input constraint")
                .await);
        }
        chars.splice(index as usize..index as usize, inserted);
        Self::store(jvm, this, chars, index + len).await
    }

    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, len: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::delete({this:?}, {index}, {len})");
        let mut chars = Self::chars(jvm, &this).await?;
        if index < 0 || index as usize > chars.len() || len < 0 {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid deletion range").await);
        }
        if len as usize > chars.len() - index as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Deletion exceeds text length").await);
        }
        chars.drain(index as usize..index as usize + len as usize);
        Self::store(jvm, this, chars, index).await
    }

    async fn key_notify(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, kind: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::keyNotify({this:?}, {kind}, {key})");
        if kind != KeyboardEventType::KeyPressed as i32 {
            return Ok(false);
        }
        if key == WIPIKeyCode::CLEAR as i32 {
            let cursor: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "m_cPos", "I").await?;
            if cursor <= 0 {
                return Ok(true);
            }
            let chars = Self::chars(jvm, &this).await?;
            let preceding = &chars[..cursor as usize];
            let length = if matches!(preceding, [.., 0xd800..=0xdbff, 0xdc00..=0xdfff]) {
                2
            } else {
                1
            };
            let _: () = jvm
                .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "delete", "(II)V", (cursor - length, length))
                .await?;
            return Ok(true);
        }
        if !(WIPIKeyCode::NUM0 as i32..=WIPIKeyCode::NUM9 as i32).contains(&key) && key != WIPIKeyCode::STAR as i32 && key != WIPIKeyCode::HASH as i32
        {
            return Ok(false);
        }
        let current: ClassInstanceRef<String> = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
            .await?;
        let initial = JavaLangString::to_rust_string(jvm, &current).await?;
        let Some(text) = context.system().platform().text_input(&initial).await else {
            return Ok(true);
        };
        let chars = text.encode_utf16().collect::<Vec<_>>();
        let constraint = jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "constraint", "I").await?;
        if !Self::accepts(constraint, &chars) {
            return Ok(true);
        }
        let value = JavaLangString::from_utf16(jvm, chars).await?;
        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (value,))
            .await?;
        Ok(true)
    }

    async fn set_font(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setFont({this:?}, {font:?})");
        let font = if font.is_null() {
            jvm.invoke_static("org/kwis/msp/lcdui/Font", "getDefaultFont", "()Lorg/kwis/msp/lcdui/Font;", ())
                .await?
        } else {
            font
        };
        jvm.put_field(&mut this, "org/kwis/msp/lwc/TextComponent", "f", "Lorg/kwis/msp/lcdui/Font;", font)
            .await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getFont({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lwc/TextComponent", "f", "Lorg/kwis/msp/lcdui/Font;")
            .await
    }

    async fn text_lines<T: Sync>(
        jvm: &Jvm,
        this: &ClassInstanceRef<T>,
        wrap_width: Option<i32>,
        font: &ClassInstanceRef<Font>,
    ) -> JvmResult<(Vec<Vec<u16>>, i32)> {
        let mut chars = Self::chars(jvm, this).await?;
        if jvm.get_field::<i32>(this, "org/kwis/msp/lwc/TextComponent", "constraint", "I").await? == 2 {
            chars.fill(u16::from(b'*'));
        }
        let Some(width) = wrap_width else {
            let text = JavaLangString::from_utf16(jvm, chars.clone()).await?;
            let width = jvm
                .invoke_virtual(font, "org/kwis/msp/lcdui/Font", "stringWidth", "(Ljava/lang/String;)I", (text,))
                .await?;
            return Ok((vec![chars], width));
        };
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut line_width = 0;
        let mut widest = 0;
        for ch in char::decode_utf16(chars).map(|ch| ch.unwrap_or(char::REPLACEMENT_CHARACTER)) {
            let mut units = [0; 2];
            let glyph = ch.encode_utf16(&mut units);
            let advance: i32 = if glyph.len() == 1 {
                jvm.invoke_virtual(font, "org/kwis/msp/lcdui/Font", "charWidth", "(C)I", (glyph[0],))
                    .await?
            } else {
                let text = JavaLangString::from_utf16(jvm, glyph.to_vec()).await?;
                jvm.invoke_virtual(font, "org/kwis/msp/lcdui/Font", "stringWidth", "(Ljava/lang/String;)I", (text,))
                    .await?
            };
            let newline = ch == '\n';
            if newline || width >= 0 && line_width + advance > width - 4 && !line.is_empty() {
                widest = widest.max(line_width);
                lines.push(core::mem::take(&mut line));
                line_width = 0;
                if newline {
                    continue;
                }
            }
            line.extend_from_slice(glyph);
            line_width += advance;
        }
        lines.push(line);
        Ok((lines, widest.max(line_width)))
    }

    async fn calc_preferred_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::calcPreferredSize({this:?}, {width})");
        let (preferred_width, preferred_height) = Self::measure_text(jvm, &this, None).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefW", "I", preferred_width)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefH", "I", preferred_height)
            .await
    }

    pub async fn measure_text<T: Sync>(jvm: &Jvm, this: &ClassInstanceRef<T>, wrap_width: Option<i32>) -> JvmResult<(i32, i32)> {
        let font: ClassInstanceRef<Font> = jvm
            .get_field(this, "org/kwis/msp/lwc/TextComponent", "f", "Lorg/kwis/msp/lcdui/Font;")
            .await?;
        let line_height: i32 = jvm.invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "getHeight", "()I", ()).await?;
        let (lines, widest) = Self::text_lines(jvm, this, wrap_width, &font).await?;
        Ok((widest.max(line_height) + 4, lines.len() as i32 * line_height + 4))
    }

    async fn paint_content(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::paintContent({this:?}, {g:?})");
        Self::draw_text(jvm, this, g, false).await
    }

    pub async fn draw_text<T: Sync>(jvm: &Jvm, this: ClassInstanceRef<T>, g: ClassInstanceRef<Graphics>, wrap: bool) -> JvmResult<()> {
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/Component",
                "paintContent",
                "(Lorg/kwis/msp/lcdui/Graphics;)V",
                (g.clone(),),
            )
            .await?;
        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?;
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        let font: ClassInstanceRef<Font> = jvm
            .get_field(&this, "org/kwis/msp/lwc/TextComponent", "f", "Lorg/kwis/msp/lcdui/Font;")
            .await?;
        let line_height: i32 = jvm.invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "getHeight", "()I", ()).await?;
        let (lines, _) = Self::text_lines(jvm, &this, wrap.then_some(width), &font).await?;
        let color: i32 = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "getForeground", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;
        let _: () = jvm
            .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "setFont", "(Lorg/kwis/msp/lcdui/Font;)V", (font,))
            .await?;
        let _: () = jvm
            .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "drawRect", "(IIII)V", (0, 0, width - 1, height - 1))
            .await?;
        for (row, line) in lines.into_iter().enumerate() {
            let y = 2 + row as i32 * line_height;
            if y >= height {
                break;
            }
            let text = JavaLangString::from_utf16(jvm, line).await?;
            let _: () = jvm
                .invoke_virtual(
                    &g,
                    "org/kwis/msp/lcdui/Graphics",
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (text, 2, y, 4),
                )
                .await?;
        }
        Ok(())
    }
}
