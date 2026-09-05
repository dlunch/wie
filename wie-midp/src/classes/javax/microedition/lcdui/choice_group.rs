use alloc::{
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Font, Graphics, Image, Item},
    net::wie::{ChoiceElement, MIDPKeyCode},
};

const INDICATOR_SIZE: i32 = 10;
const CONTROL_GAP: i32 = 4;
const IMAGE_TEXT_GAP: i32 = 3;
const ROW_VERTICAL_INSET: i32 = 1;
const POPUP_INSET: i32 = 2;
const TEXT_COLOR: i32 = 0x17212b;
const CONTROL_COLOR: i32 = 0x596773;
const SELECTED_COLOR: i32 = 0x2f7eb8;
const HIGHLIGHT_BACKGROUND: i32 = 0xdcecf7;
const POPUP_BACKGROUND: i32 = 0xf8fafb;
const LEFT_TOP: i32 = 4 | 16;

struct ElementContent {
    text: RustString,
    display_image: ClassInstanceRef<Image>,
    image_width: i32,
    image_height: i32,
    font: ClassInstanceRef<Font>,
    font_height: i32,
}

// class javax.microedition.lcdui.ChoiceGroup
pub struct ChoiceGroup;

impl ChoiceGroup {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/ChoiceGroup",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec!["javax/microedition/lcdui/Choice"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                    Self::init_with_elements,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "(I)Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getImage",
                    "(I)Ljavax/microedition/lcdui/Image;",
                    Self::get_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
                    Self::append,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "insert",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    Self::insert,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("delete", "(I)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deleteAll", "()V", Self::delete_all, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "set",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    Self::set,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("isSelected", "(I)Z", Self::is_selected, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedIndex", "()I", Self::get_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedFlags", "([Z)I", Self::get_selected_flags, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSelectedIndex", "(IZ)V", Self::set_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setSelectedFlags", "([Z)V", Self::set_selected_flags, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFitPolicy", "(I)V", Self::set_fit_policy, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFitPolicy", "()I", Self::get_fit_policy, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setFont",
                    "(ILjavax/microedition/lcdui/Font;)V",
                    Self::set_font,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getFont", "(I)Ljavax/microedition/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
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
                JavaMethodProto::new(
                    "getFocusContentBounds",
                    "(I)[I",
                    Self::get_focus_content_bounds,
                    MethodAccessFlags::empty(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("choiceType", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("elements", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("fitPolicy", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("highlightedIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("popupOpen", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        choice_type: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.ChoiceGroup::<init>({this:?}, {label:?}, {choice_type})");

        let string_elements = jvm.instantiate_array("[Ljava/lang/String;", 0).await?;
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/ChoiceGroup",
            "<init>",
            "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
            (label, choice_type, string_elements, None),
        )
        .await
    }

    async fn init_with_elements(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        choice_type: i32,
        string_elements: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
        image_elements: ClassInstanceRef<Array<ClassInstanceRef<Image>>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.ChoiceGroup::<init>({this:?}, {label:?}, {choice_type}, {string_elements:?}, {image_elements:?})");

        if !matches!(choice_type, 1 | 2 | 4) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid ChoiceGroup type").await);
        }
        if string_elements.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "String element array is null").await);
        }

        let element_count = jvm.array_length(&string_elements).await?;
        if !image_elements.is_null() && jvm.array_length(&image_elements).await? != element_count {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Choice element array lengths differ")
                .await);
        }
        let strings: Vec<ClassInstanceRef<String>> = jvm.load_array(&string_elements, 0, element_count).await?;
        if strings.iter().any(ClassInstanceRef::is_null) {
            return Err(jvm.exception("java/lang/NullPointerException", "Choice string is null").await);
        }
        let images: Vec<ClassInstanceRef<Image>> = if image_elements.is_null() {
            vec![None.into(); element_count]
        } else {
            jvm.load_array(&image_elements, 0, element_count).await?
        };

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        let elements: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "choiceType", "I", choice_type).await?;
        jvm.put_field(&mut this, "elements", "Ljava/util/Vector;", elements.clone()).await?;
        jvm.put_field(&mut this, "fitPolicy", "I", 0).await?;
        jvm.put_field(&mut this, "highlightedIndex", "I", -1).await?;
        jvm.put_field(&mut this, "popupOpen", "Z", false).await?;

        for (index, (text, image)) in strings.into_iter().zip(images).enumerate() {
            let element = jvm
                .new_class(
                    "net/wie/ChoiceElement",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;Z)V",
                    (text, image, choice_type != 2 && index == 0),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&elements, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (element,))
                .await?;
        }

        Self::normalize_highlight(jvm, &mut this).await?;

        Ok(())
    }

    async fn element_at(jvm: &Jvm, this: &ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<ChoiceElement>> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Err(jvm
                .exception("java/lang/IndexOutOfBoundsException", "Choice index is out of bounds")
                .await);
        }

        jvm.invoke_virtual(&elements, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    async fn selected_index(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        for index in 0..size {
            let element = Self::element_at(jvm, this, index).await?;
            if jvm.get_field(&element, "selected", "Z").await? {
                return Ok(index);
            }
        }
        Ok(-1)
    }

    async fn select_only(jvm: &Jvm, this: &ClassInstanceRef<Self>, selected_index: i32) -> JvmResult<bool> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        let mut changed = false;
        for index in 0..size {
            let mut element = Self::element_at(jvm, this, index).await?;
            let selected: bool = jvm.get_field(&element, "selected", "Z").await?;
            let new_selected = index == selected_index;
            if selected != new_selected {
                jvm.put_field(&mut element, "selected", "Z", new_selected).await?;
                changed = true;
            }
        }
        Ok(changed)
    }

    async fn normalize_highlight(jvm: &Jvm, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            jvm.put_field(this, "highlightedIndex", "I", -1).await?;
            return jvm.put_field(this, "popupOpen", "Z", false).await;
        }

        let highlighted_index: i32 = jvm.get_field(this, "highlightedIndex", "I").await?;
        let highlighted_index = if highlighted_index < 0 {
            let choice_type: i32 = jvm.get_field(this, "choiceType", "I").await?;
            if choice_type == 2 { 0 } else { Self::selected_index(jvm, this).await? }
        } else {
            highlighted_index.min(size - 1)
        };
        jvm.put_field(this, "highlightedIndex", "I", highlighted_index).await
    }

    async fn element_content(jvm: &Jvm, element: &ClassInstanceRef<ChoiceElement>) -> JvmResult<ElementContent> {
        let text: ClassInstanceRef<String> = jvm.get_field(element, "text", "Ljava/lang/String;").await?;
        let text = JavaLangString::to_rust_string(jvm, &text).await?;
        let display_image: ClassInstanceRef<Image> = jvm.get_field(element, "displayImage", "Ljavax/microedition/lcdui/Image;").await?;
        let (image_width, image_height) = if display_image.is_null() {
            (0, 0)
        } else {
            (
                jvm.invoke_virtual(&display_image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
                    .await?,
                jvm.invoke_virtual(&display_image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
                    .await?,
            )
        };
        let mut font: ClassInstanceRef<Font> = jvm.get_field(element, "font", "Ljavax/microedition/lcdui/Font;").await?;
        if font.is_null() {
            font = jvm
                .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await?;
        }
        let font_height: i32 = jvm.invoke_virtual(&font, "javax/microedition/lcdui/Font", "getHeight", "()I", ()).await?;

        Ok(ElementContent {
            text,
            display_image,
            image_width,
            image_height,
            font,
            font_height: font_height.max(1),
        })
    }

    async fn font_text_width(jvm: &Jvm, font: &ClassInstanceRef<Font>, text: &str) -> JvmResult<i32> {
        let text = JavaLangString::from_rust_string(jvm, text).await?;
        jvm.invoke_virtual(font, "javax/microedition/lcdui/Font", "stringWidth", "(Ljava/lang/String;)I", (text,))
            .await
    }

    async fn preferred_text_width(jvm: &Jvm, content: &ElementContent) -> JvmResult<i32> {
        let mut width = 0;
        for line in content.text.split('\n') {
            width = width.max(Self::font_text_width(jvm, &content.font, line).await?);
        }
        Ok(width)
    }

    async fn minimum_text_width(jvm: &Jvm, content: &ElementContent, fit_policy: i32) -> JvmResult<i32> {
        if fit_policy == 2 {
            return Self::preferred_text_width(jvm, content).await;
        }

        let mut width = 0;
        for character in content.text.chars().filter(|character| *character != '\n') {
            width = width.max(Self::font_text_width(jvm, &content.font, &character.to_string()).await?);
        }
        Ok(width)
    }

    async fn wrap_text(jvm: &Jvm, content: &ElementContent, maximum_width: Option<i32>) -> JvmResult<Vec<RustString>> {
        let mut lines = Vec::new();
        for paragraph in content.text.split('\n') {
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
            let mut widths = Vec::with_capacity(characters.len());
            for character in &characters {
                widths.push(Self::font_text_width(jvm, &content.font, &character.to_string()).await?);
            }

            let mut start = 0;
            while start < characters.len() {
                let mut end = start;
                let mut width = 0;
                let mut word_boundary = None;
                while end < characters.len() {
                    let character_width = widths[end];
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
        Ok(lines)
    }

    fn image_and_text_width(content: &ElementContent, text_width: i32) -> i32 {
        content.image_width
            + if content.image_width > 0 && !content.text.is_empty() {
                IMAGE_TEXT_GAP
            } else {
                0
            }
            + text_width
    }

    async fn element_row_height(jvm: &Jvm, content: &ElementContent, width: i32, fit_policy: i32) -> JvmResult<i32> {
        let text_width = (width - INDICATOR_SIZE - CONTROL_GAP - Self::image_and_text_width(content, 0)).max(1);
        let lines = Self::wrap_text(jvm, content, (fit_policy != 2).then_some(text_width)).await?;
        Ok((lines.len() as i32 * content.font_height).max(content.image_height).max(INDICATOR_SIZE) + ROW_VERTICAL_INSET * 2)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_element(
        jvm: &Jvm,
        graphics: &ClassInstanceRef<Graphics>,
        content: &ElementContent,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        choice_type: i32,
        fit_policy: i32,
        selected: bool,
        highlighted: bool,
        popup_closed: bool,
    ) -> JvmResult<()> {
        if highlighted {
            let _: () = jvm
                .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (HIGHLIGHT_BACKGROUND,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (x, y, width.max(0), height.max(0)),
                )
                .await?;
        }

        let control_on_left = !popup_closed;
        if control_on_left {
            let indicator_x = x;
            let indicator_y = y + (height - INDICATOR_SIZE) / 2;
            let _: () = jvm
                .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (CONTROL_COLOR,))
                .await?;
            if choice_type == 2 {
                let _: () = jvm
                    .invoke_virtual(
                        graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawRect",
                        "(IIII)V",
                        (indicator_x, indicator_y, INDICATOR_SIZE - 1, INDICATOR_SIZE - 1),
                    )
                    .await?;
                if selected {
                    let _: () = jvm
                        .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (SELECTED_COLOR,))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(
                            graphics,
                            "javax/microedition/lcdui/Graphics",
                            "fillRect",
                            "(IIII)V",
                            (indicator_x + 2, indicator_y + 2, INDICATOR_SIZE - 4, INDICATOR_SIZE - 4),
                        )
                        .await?;
                }
            } else {
                let _: () = jvm
                    .invoke_virtual(
                        graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawArc",
                        "(IIIIII)V",
                        (indicator_x, indicator_y, INDICATOR_SIZE - 1, INDICATOR_SIZE - 1, 0, 360),
                    )
                    .await?;
                if selected {
                    let _: () = jvm
                        .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (SELECTED_COLOR,))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(
                            graphics,
                            "javax/microedition/lcdui/Graphics",
                            "fillArc",
                            "(IIIIII)V",
                            (indicator_x + 3, indicator_y + 3, INDICATOR_SIZE - 6, INDICATOR_SIZE - 6, 0, 360),
                        )
                        .await?;
                }
            }
        }

        if popup_closed {
            let arrow_x = x + width - INDICATOR_SIZE / 2 - 1;
            let arrow_y = y + height / 2;
            let _: () = jvm
                .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (CONTROL_COLOR,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawLine",
                    "(IIII)V",
                    (arrow_x - 3, arrow_y - 2, arrow_x, arrow_y + 1),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawLine",
                    "(IIII)V",
                    (arrow_x, arrow_y + 1, arrow_x + 3, arrow_y - 2),
                )
                .await?;
        }

        let mut content_x = x + if control_on_left { INDICATOR_SIZE + CONTROL_GAP } else { 0 };
        let trailing_control_width = if popup_closed { INDICATOR_SIZE + CONTROL_GAP } else { 0 };
        let available_right = x + width - trailing_control_width;
        if !content.display_image.is_null() && content.image_width > 0 && content.image_height > 0 {
            let image_y = y + (height - content.image_height) / 2;
            let _: () = jvm
                .invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawImage",
                    "(Ljavax/microedition/lcdui/Image;III)V",
                    (content.display_image.clone(), content_x, image_y, LEFT_TOP),
                )
                .await?;
            content_x += content.image_width;
            if !content.text.is_empty() {
                content_x += IMAGE_TEXT_GAP;
            }
        }

        let text_width = (available_right - content_x).max(1);
        let lines = Self::wrap_text(jvm, content, (fit_policy != 2).then_some(text_width)).await?;
        let text_height = lines.len() as i32 * content.font_height;
        let text_y = y + (height - text_height) / 2;
        let _: () = jvm
            .invoke_virtual(
                graphics,
                "javax/microedition/lcdui/Graphics",
                "setFont",
                "(Ljavax/microedition/lcdui/Font;)V",
                (content.font.clone(),),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TEXT_COLOR,))
            .await?;
        for (line_index, line) in lines.iter().enumerate() {
            let line_y = text_y + line_index as i32 * content.font_height;
            if line_y >= y + height {
                break;
            }
            let line = JavaLangString::from_rust_string(jvm, line).await?;
            let _: () = jvm
                .invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (line, content_x, line_y, LEFT_TOP),
                )
                .await?;
        }

        Ok(())
    }

    async fn size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await
    }

    async fn get_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<String>> {
        let element = Self::element_at(jvm, &this, index).await?;
        jvm.get_field(&element, "text", "Ljava/lang/String;").await
    }

    async fn get_image(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Image>> {
        let element = Self::element_at(jvm, &this, index).await?;
        jvm.get_field(&element, "image", "Ljavax/microedition/lcdui/Image;").await
    }

    async fn append(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let index: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/ChoiceGroup",
                "insert",
                "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                (index, text, image),
            )
            .await?;
        Ok(index)
    }

    async fn insert(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index > size {
            return Err(jvm
                .exception("java/lang/IndexOutOfBoundsException", "Choice insert index is out of bounds")
                .await);
        }
        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Choice string is null").await);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let element = jvm
            .new_class(
                "net/wie/ChoiceElement",
                "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;Z)V",
                (text, image, choice_type != 2 && size == 0),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &elements,
                "java/util/Vector",
                "insertElementAt",
                "(Ljava/lang/Object;I)V",
                (element, index),
            )
            .await?;

        let highlighted_index: i32 = jvm.get_field(&this, "highlightedIndex", "I").await?;
        jvm.put_field(
            &mut this,
            "highlightedIndex",
            "I",
            if highlighted_index < 0 {
                -1
            } else if index <= highlighted_index {
                highlighted_index + 1
            } else {
                highlighted_index
            },
        )
        .await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn delete(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        let removed = Self::element_at(jvm, &this, index).await?;
        let was_selected: bool = jvm.get_field(&removed, "selected", "Z").await?;
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let _: ClassInstanceRef<ChoiceElement> = jvm
            .invoke_virtual(&elements, "java/util/Vector", "remove", "(I)Ljava/lang/Object;", (index,))
            .await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        if was_selected && choice_type != 2 && size > 0 {
            let mut replacement = Self::element_at(jvm, &this, index.min(size - 1)).await?;
            jvm.put_field(&mut replacement, "selected", "Z", true).await?;
        }

        let highlighted_index: i32 = jvm.get_field(&this, "highlightedIndex", "I").await?;
        let highlighted_index = if highlighted_index > index {
            highlighted_index - 1
        } else {
            highlighted_index
        };
        jvm.put_field(&mut this, "highlightedIndex", "I", highlighted_index).await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn delete_all(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let _: () = jvm.invoke_virtual(&elements, "java/util/Vector", "removeAllElements", "()V", ()).await?;
        jvm.put_field(&mut this, "highlightedIndex", "I", -1).await?;
        jvm.put_field(&mut this, "popupOpen", "Z", false).await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn set(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        let mut element = Self::element_at(jvm, &this, index).await?;
        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Choice string is null").await);
        }
        let display_image: ClassInstanceRef<Image> = if image.is_null() {
            None.into()
        } else {
            jvm.invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                (image.clone(),),
            )
            .await?
        };
        jvm.put_field(&mut element, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut element, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut element, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn is_selected(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<bool> {
        let element = Self::element_at(jvm, &this, index).await?;
        jvm.get_field(&element, "selected", "Z").await
    }

    async fn get_selected_index(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        if choice_type == 2 {
            return Ok(-1);
        }
        Self::selected_index(jvm, &this).await
    }

    async fn get_selected_flags(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut selected_array: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<i32> {
        if selected_array.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Selection array is null").await);
        }
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        let array_length = jvm.array_length(&selected_array).await?;
        if array_length < size as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Selection array is too short").await);
        }

        let mut selected_count = 0;
        let mut flags = vec![false; array_length];
        for index in 0..size {
            let element = Self::element_at(jvm, &this, index).await?;
            let selected: bool = jvm.get_field(&element, "selected", "Z").await?;
            flags[index as usize] = selected;
            selected_count += i32::from(selected);
        }
        jvm.store_array(&mut selected_array, 0, flags).await?;
        Ok(selected_count)
    }

    async fn set_selected_index(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        selected: bool,
    ) -> JvmResult<()> {
        let mut target = Self::element_at(jvm, &this, index).await?;
        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let changed = if choice_type == 2 {
            let old_selected: bool = jvm.get_field(&target, "selected", "Z").await?;
            if old_selected != selected {
                jvm.put_field(&mut target, "selected", "Z", selected).await?;
            }
            old_selected != selected
        } else if selected {
            Self::select_only(jvm, &this, index).await?
        } else {
            false
        };
        Self::normalize_highlight(jvm, &mut this).await?;
        if changed {
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (choice_type == 4,))
                .await?;
        }
        Ok(())
    }

    async fn set_selected_flags(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        selected_array: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<()> {
        if selected_array.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Selection array is null").await);
        }
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if jvm.array_length(&selected_array).await? < size as usize {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Selection array is too short").await);
        }
        let flags: Vec<bool> = jvm.load_array(&selected_array, 0, size as usize).await?;
        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let mut changed = false;
        if choice_type == 2 {
            for (index, selected) in flags.into_iter().enumerate() {
                let mut element = Self::element_at(jvm, &this, index as i32).await?;
                let old_selected: bool = jvm.get_field(&element, "selected", "Z").await?;
                if old_selected != selected {
                    jvm.put_field(&mut element, "selected", "Z", selected).await?;
                    changed = true;
                }
            }
        } else if size > 0 {
            let selected_index = flags.iter().position(|selected| *selected).unwrap_or(0) as i32;
            changed = Self::select_only(jvm, &this, selected_index).await?;
        }
        Self::normalize_highlight(jvm, &mut this).await?;
        if changed {
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (choice_type == 4,))
                .await?;
        }
        Ok(())
    }

    async fn set_fit_policy(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fit_policy: i32) -> JvmResult<()> {
        if !(0..=2).contains(&fit_policy) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Choice fit policy").await);
        }
        jvm.put_field(&mut this, "fitPolicy", "I", fit_policy).await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn get_fit_policy(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "fitPolicy", "I").await
    }

    async fn set_font(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        font: ClassInstanceRef<Font>,
    ) -> JvmResult<()> {
        let mut element = Self::element_at(jvm, &this, index).await?;
        jvm.put_field(&mut element, "font", "Ljavax/microedition/lcdui/Font;", font).await?;
        Self::normalize_highlight(jvm, &mut this).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn get_font(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Font>> {
        let element = Self::element_at(jvm, &this, index).await?;
        let font: ClassInstanceRef<Font> = jvm.get_field(&element, "font", "Ljavax/microedition/lcdui/Font;").await?;
        if font.is_null() {
            jvm.invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await
        } else {
            Ok(font)
        }
    }

    async fn minimum_content_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(0);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let fit_policy: i32 = jvm.get_field(&this, "fitPolicy", "I").await?;
        let mut width = 0;
        for index in 0..size {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            let text_width = Self::minimum_text_width(jvm, &content, fit_policy).await?;
            width = width.max(INDICATOR_SIZE + CONTROL_GAP + Self::image_and_text_width(&content, text_width));
        }
        Ok(width + if choice_type == 4 { POPUP_INSET * 2 } else { 0 })
    }

    async fn minimum_content_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(0);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let popup_open: bool = jvm.get_field(&this, "popupOpen", "Z").await?;
        let selected_index = if choice_type == 4 && !popup_open {
            Self::selected_index(jvm, &this).await?
        } else {
            0
        };
        let end_index = if choice_type == 4 && !popup_open { selected_index + 1 } else { size };
        let mut height = 0;
        for index in selected_index..end_index {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            let explicit_lines = content.text.split('\n').count() as i32;
            height += (explicit_lines * content.font_height).max(content.image_height).max(INDICATOR_SIZE) + ROW_VERTICAL_INSET * 2;
        }
        Ok(height + if choice_type == 4 { POPUP_INSET * 2 } else { 0 })
    }

    async fn preferred_content_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(0);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let mut width = 0;
        for index in 0..size {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            let text_width = Self::preferred_text_width(jvm, &content).await?;
            width = width.max(INDICATOR_SIZE + CONTROL_GAP + Self::image_and_text_width(&content, text_width));
        }
        Ok(width + if choice_type == 4 { POPUP_INSET * 2 } else { 0 })
    }

    async fn preferred_content_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(0);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let fit_policy: i32 = jvm.get_field(&this, "fitPolicy", "I").await?;
        let popup_open: bool = jvm.get_field(&this, "popupOpen", "Z").await?;
        let selected_index = if choice_type == 4 && !popup_open {
            Self::selected_index(jvm, &this).await?
        } else {
            0
        };
        let end_index = if choice_type == 4 && !popup_open { selected_index + 1 } else { size };
        let row_width = (width - if choice_type == 4 { POPUP_INSET * 2 } else { 0 }).max(0);
        let mut height = 0;
        for index in selected_index..end_index {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            height += Self::element_row_height(jvm, &content, row_width, fit_policy).await?;
        }
        Ok(height + if choice_type == 4 { POPUP_INSET * 2 } else { 0 })
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        focused: bool,
    ) -> JvmResult<()> {
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(());
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let fit_policy: i32 = jvm.get_field(&this, "fitPolicy", "I").await?;
        let highlighted_index: i32 = jvm.get_field(&this, "highlightedIndex", "I").await?;
        let popup_open: bool = jvm.get_field(&this, "popupOpen", "Z").await?;
        let popup_closed = choice_type == 4 && !popup_open;

        let (row_x, mut row_y, row_width) = if choice_type == 4 {
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (POPUP_BACKGROUND,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (x, y, width, height),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (CONTROL_COLOR,))
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
            (x + POPUP_INSET, y + POPUP_INSET, (width - POPUP_INSET * 2).max(0))
        } else {
            (x, y, width)
        };

        let start_index = if popup_closed { Self::selected_index(jvm, &this).await? } else { 0 };
        let end_index = if popup_closed { start_index + 1 } else { size };
        for index in start_index..end_index {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            let row_height = Self::element_row_height(jvm, &content, row_width, fit_policy).await?;
            let selected: bool = jvm.get_field(&element, "selected", "Z").await?;
            Self::paint_element(
                jvm,
                &graphics,
                &content,
                row_x,
                row_y,
                row_width,
                row_height,
                choice_type,
                fit_policy,
                selected,
                index == highlighted_index && (focused || popup_open),
                popup_closed,
            )
            .await?;
            row_y += row_height;
        }

        Ok(())
    }

    async fn get_focus_content_bounds(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        width: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        let highlighted_index: i32 = jvm.get_field(&this, "highlightedIndex", "I").await?;
        if highlighted_index < 0 {
            return Ok(None.into());
        }
        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let popup_open: bool = jvm.get_field(&this, "popupOpen", "Z").await?;
        let popup_closed = choice_type == 4 && !popup_open;
        let active_index = if popup_closed {
            Self::selected_index(jvm, &this).await?
        } else {
            highlighted_index
        };
        let inset = if choice_type == 4 { POPUP_INSET } else { 0 };
        let fit_policy: i32 = jvm.get_field(&this, "fitPolicy", "I").await?;
        let mut top = inset;
        let mut bottom = inset;
        for index in if popup_closed { active_index } else { 0 }..=active_index {
            let element = Self::element_at(jvm, &this, index).await?;
            let content = Self::element_content(jvm, &element).await?;
            top = bottom;
            bottom += Self::element_row_height(jvm, &content, (width - inset * 2).max(0), fit_policy).await?;
        }
        let mut bounds = jvm.instantiate_array("[I", 2).await?;
        jvm.store_array(&mut bounds, 0, [top, bottom]).await?;
        Ok(bounds.into())
    }

    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        if jvm.invoke_virtual::<_, i32>(&elements, "java/util/Vector", "size", "()I", ()).await? > 0 {
            return Ok(true);
        }
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ()).await
    }

    async fn handle_item_key(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        let elements: ClassInstanceRef<Vector> = jvm.get_field(&this, "elements", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&elements, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(0);
        }

        let choice_type: i32 = jvm.get_field(&this, "choiceType", "I").await?;
        let highlighted_index: i32 = jvm.get_field(&this, "highlightedIndex", "I").await?;
        if choice_type == 4 {
            let popup_open: bool = jvm.get_field(&this, "popupOpen", "Z").await?;
            if !popup_open {
                if key != MIDPKeyCode::FIRE as i32 {
                    return Ok(0);
                }
                let selected_index = Self::selected_index(jvm, &this).await?;
                jvm.put_field(&mut this, "highlightedIndex", "I", selected_index).await?;
                jvm.put_field(&mut this, "popupOpen", "Z", true).await?;
                Self::normalize_highlight(jvm, &mut this).await?;
                let _: () = jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
                    .await?;
                return Ok(Item::INPUT_HANDLED);
            }

            if key == MIDPKeyCode::UP as i32 || key == MIDPKeyCode::DOWN as i32 {
                let new_highlight = if key == MIDPKeyCode::UP as i32 {
                    highlighted_index.saturating_sub(1).max(0)
                } else {
                    highlighted_index.saturating_add(1).min(size - 1)
                };
                if new_highlight != highlighted_index {
                    jvm.put_field(&mut this, "highlightedIndex", "I", new_highlight).await?;
                    let _: () = jvm
                        .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
                        .await?;
                }
                return Ok(Item::INPUT_HANDLED);
            }
            if key == MIDPKeyCode::FIRE as i32 {
                let changed = Self::select_only(jvm, &this, highlighted_index).await?;
                jvm.put_field(&mut this, "popupOpen", "Z", false).await?;
                Self::normalize_highlight(jvm, &mut this).await?;
                let _: () = jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
                    .await?;
                return Ok(Item::INPUT_HANDLED | if changed { Item::INPUT_CHANGED } else { 0 });
            }
            if key == MIDPKeyCode::CLEAR as i32 {
                let selected_index = Self::selected_index(jvm, &this).await?;
                jvm.put_field(&mut this, "highlightedIndex", "I", selected_index).await?;
                jvm.put_field(&mut this, "popupOpen", "Z", false).await?;
                Self::normalize_highlight(jvm, &mut this).await?;
                let _: () = jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
                    .await?;
                return Ok(Item::INPUT_HANDLED);
            }
            return Ok(0);
        }

        if key == MIDPKeyCode::UP as i32 || key == MIDPKeyCode::DOWN as i32 {
            let new_highlight = if key == MIDPKeyCode::UP as i32 {
                highlighted_index - 1
            } else {
                highlighted_index + 1
            };
            if new_highlight < 0 || new_highlight >= size {
                return Ok(0);
            }
            jvm.put_field(&mut this, "highlightedIndex", "I", new_highlight).await?;
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
                .await?;
            return Ok(Item::INPUT_HANDLED);
        }
        if key != MIDPKeyCode::FIRE as i32 {
            return Ok(0);
        }

        let mut element = Self::element_at(jvm, &this, highlighted_index).await?;
        let selected: bool = jvm.get_field(&element, "selected", "Z").await?;
        let changed = if choice_type == 2 {
            jvm.put_field(&mut element, "selected", "Z", !selected).await?;
            true
        } else if selected {
            false
        } else {
            Self::select_only(jvm, &this, highlighted_index).await?
        };
        if changed {
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
                .await?;
        }
        Ok(Item::INPUT_HANDLED | if changed { Item::INPUT_CHANGED } else { 0 })
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::lcdui::{ChoiceGroup, Graphics, Image},
            net::wie::MIDPKeyCode,
        },
        get_protos,
    };

    async fn choice_with_elements(
        jvm: &Jvm,
        choice_type: i32,
        texts: &[&str],
        images: Option<&[ClassInstanceRef<Image>]>,
    ) -> JvmResult<ClassInstanceRef<ChoiceGroup>> {
        let mut string_array: ClassInstanceRef<Array<ClassInstanceRef<String>>> =
            jvm.instantiate_array("[Ljava/lang/String;", texts.len()).await?.into();
        let mut strings = Vec::with_capacity(texts.len());
        for text in texts {
            strings.push(ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(jvm, text).await?));
        }
        jvm.store_array(&mut string_array, 0, strings).await?;

        let image_array: ClassInstanceRef<Array<ClassInstanceRef<Image>>> = if let Some(images) = images {
            let mut image_array: ClassInstanceRef<Array<ClassInstanceRef<Image>>> =
                jvm.instantiate_array("[Ljavax/microedition/lcdui/Image;", images.len()).await?.into();
            jvm.store_array(&mut image_array, 0, images.to_vec()).await?;
            image_array
        } else {
            None.into()
        };

        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/ChoiceGroup",
                "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                (None, choice_type, string_array, image_array),
            )
            .await?
            .into())
    }

    async fn create_image_and_graphics(jvm: &Jvm, width: i32, height: i32) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;
        let graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        Ok((image, graphics))
    }

    async fn fill(jvm: &Jvm, graphics: &ClassInstanceRef<Graphics>, color: i32, width: i32, height: i32) -> JvmResult<()> {
        let _: () = jvm
            .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;
        jvm.invoke_virtual(
            graphics,
            "javax/microedition/lcdui/Graphics",
            "fillRect",
            "(IIII)V",
            (0, 0, width, height),
        )
        .await
    }

    async fn solid_image(jvm: &Jvm, color: i32) -> JvmResult<ClassInstanceRef<Image>> {
        let (image, graphics) = create_image_and_graphics(jvm, 6, 5).await?;
        fill(jvm, &graphics, color, 6, 5).await?;
        Ok(image)
    }

    async fn measure(jvm: &Jvm, choice: &ClassInstanceRef<ChoiceGroup>, width: i32) -> JvmResult<i32> {
        jvm.invoke_virtual(choice, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (width,))
            .await
    }

    async fn paint(
        jvm: &Jvm,
        choice: &ClassInstanceRef<ChoiceGroup>,
        graphics: ClassInstanceRef<Graphics>,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        jvm.invoke_virtual(
            choice,
            "javax/microedition/lcdui/Item",
            "paintItem",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, 0, 0, width, height, false),
        )
        .await
    }

    async fn handle_key(jvm: &Jvm, choice: &ClassInstanceRef<ChoiceGroup>, key: MIDPKeyCode) -> JvmResult<i32> {
        jvm.invoke_virtual(choice, "javax/microedition/lcdui/Item", "handleItemKey", "(I)I", (key as i32,))
            .await
    }

    async fn has_color(jvm: &Jvm, image: &ClassInstanceRef<Image>, width: i32, height: i32, color: (u8, u8, u8)) -> JvmResult<bool> {
        let pixels = Image::image(jvm, image).await?;
        for y in 0..height {
            for x in 0..width {
                let pixel = pixels.get_pixel(x, y);
                if (pixel.r, pixel.g, pixel.b) == color {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    #[test]
    fn choice_interface_selection_survives_mutation() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let choice = choice_with_elements(&jvm, 1, &["first", "second"], None).await?;
            assert!(jvm.is_instance(&**choice, "javax/microedition/lcdui/Choice"));
            let _: () = jvm
                .invoke_virtual(&choice, "javax/microedition/lcdui/Choice", "setSelectedIndex", "(IZ)V", (1, true))
                .await?;
            let text = JavaLangString::from_rust_string(&jvm, "inserted").await?;
            let _: () = jvm
                .invoke_virtual(
                    &choice,
                    "javax/microedition/lcdui/Choice",
                    "insert",
                    "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                    (0, text, None),
                )
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                2
            );
            let _: () = jvm
                .invoke_virtual(&choice, "javax/microedition/lcdui/Choice", "delete", "(I)V", (2,))
                .await?;
            let selected: i32 = jvm
                .invoke_virtual(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                .await?;
            assert!(
                (0..2).contains(&selected),
                "deleting the selected element must select a remaining element"
            );

            let multiple = choice_with_elements(&jvm, 2, &["first", "second"], None).await?;
            let mut flags: ClassInstanceRef<Array<bool>> = jvm.instantiate_array("Z", 3).await?.into();
            jvm.store_array(&mut flags, 0, [true, true, true]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &multiple,
                    "javax/microedition/lcdui/Choice",
                    "setSelectedFlags",
                    "([Z)V",
                    (flags.clone(),),
                )
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(
                    &multiple,
                    "javax/microedition/lcdui/Choice",
                    "getSelectedFlags",
                    "([Z)I",
                    (flags.clone(),)
                )
                .await?,
                2
            );
            assert_eq!(jvm.load_array::<bool>(&flags, 0, 3).await?, [true, true, false]);

            let invalid = choice_with_elements(&jvm, 3, &[], None).await;
            let Err(JavaError::JavaException(exception)) = invalid else {
                panic!("ChoiceGroup accepted IMPLICIT");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
            Ok(())
        })
    }

    #[test]
    fn choice_group_inline_keys_report_boundaries_and_selection_changes_exactly() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let exclusive = choice_with_elements(&jvm, 1, &["first", "second", "third"], None).await?;
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::UP).await?, 0);
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::FIRE).await?, 1);
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&exclusive, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                0
            );
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::FIRE).await?, 3);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&exclusive, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                1
            );
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::FIRE).await?, 1);
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::DOWN).await?, 0);
            assert_eq!(handle_key(&jvm, &exclusive, MIDPKeyCode::RIGHT).await?, 0);

            let multiple = choice_with_elements(&jvm, 2, &["first", "second"], None).await?;
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::UP).await?, 0);
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::FIRE).await?, 3);
            assert!(
                jvm.invoke_virtual::<_, bool>(&multiple, "javax/microedition/lcdui/Choice", "isSelected", "(I)Z", (0,))
                    .await?
            );
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::FIRE).await?, 3);
            assert!(
                !jvm.invoke_virtual::<_, bool>(&multiple, "javax/microedition/lcdui/Choice", "isSelected", "(I)Z", (0,))
                    .await?
            );
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::FIRE).await?, 3);
            assert!(
                jvm.invoke_virtual::<_, bool>(&multiple, "javax/microedition/lcdui/Choice", "isSelected", "(I)Z", (1,))
                    .await?
            );
            assert_eq!(handle_key(&jvm, &multiple, MIDPKeyCode::DOWN).await?, 0);

            Ok(())
        })
    }

    #[test]
    fn choice_group_popup_paints_closed_and_open_states_then_commits_or_cancels() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let red = solid_image(&jvm, 0xcc1122).await?;
            let green = solid_image(&jvm, 0x22aa55).await?;
            let blue = solid_image(&jvm, 0x2255dd).await?;
            let choice = choice_with_elements(&jvm, 4, &["red", "green", "blue"], Some(&[red, green, blue])).await?;

            let closed_height = measure(&jvm, &choice, 90).await?;
            let (closed_target, closed_graphics) = create_image_and_graphics(&jvm, 90, closed_height).await?;
            fill(&jvm, &closed_graphics, 0xffffff, 90, closed_height).await?;
            paint(&jvm, &choice, closed_graphics, 90, closed_height).await?;
            assert!(has_color(&jvm, &closed_target, 90, closed_height, (0xcc, 0x11, 0x22)).await?);
            assert!(!has_color(&jvm, &closed_target, 90, closed_height, (0x22, 0xaa, 0x55)).await?);
            assert!(!has_color(&jvm, &closed_target, 90, closed_height, (0x22, 0x55, 0xdd)).await?);

            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?, 1);
            let open_height = measure(&jvm, &choice, 90).await?;
            assert!(open_height > closed_height);
            let (open_target, open_graphics) = create_image_and_graphics(&jvm, 90, open_height).await?;
            fill(&jvm, &open_graphics, 0xffffff, 90, open_height).await?;
            paint(&jvm, &choice, open_graphics, 90, open_height).await?;
            assert!(has_color(&jvm, &open_target, 90, open_height, (0xcc, 0x11, 0x22)).await?);
            assert!(has_color(&jvm, &open_target, 90, open_height, (0x22, 0xaa, 0x55)).await?);
            assert!(has_color(&jvm, &open_target, 90, open_height, (0x22, 0x55, 0xdd)).await?);

            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::UP).await?, 1);
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                0
            );
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::CLEAR).await?, 1);
            assert_eq!(measure(&jvm, &choice, 90).await?, closed_height);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                0
            );

            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?, 1);
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::DOWN).await?, 1);
            assert_eq!(
                handle_key(&jvm, &choice, MIDPKeyCode::DOWN).await?,
                1,
                "open popup consumes DOWN at its lower bound"
            );
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?, 3);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                2
            );
            assert_eq!(measure(&jvm, &choice, 90).await?, closed_height);

            let (committed_target, committed_graphics) = create_image_and_graphics(&jvm, 90, closed_height).await?;
            fill(&jvm, &committed_graphics, 0xffffff, 90, closed_height).await?;
            paint(&jvm, &choice, committed_graphics, 90, closed_height).await?;
            assert!(!has_color(&jvm, &committed_target, 90, closed_height, (0xcc, 0x11, 0x22)).await?);
            assert!(!has_color(&jvm, &committed_target, 90, closed_height, (0x22, 0xaa, 0x55)).await?);
            assert!(has_color(&jvm, &committed_target, 90, closed_height, (0x22, 0x55, 0xdd)).await?);

            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?, 1);
            assert_eq!(
                handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?,
                1,
                "committing the selected element is a no-op"
            );
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::FIRE).await?, 1);
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::UP).await?, 1);
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::CLEAR).await?, 1);
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&choice, "javax/microedition/lcdui/Choice", "getSelectedIndex", "()I", ())
                    .await?,
                2
            );
            assert_eq!(handle_key(&jvm, &choice, MIDPKeyCode::CLEAR).await?, 0);

            Ok(())
        })
    }
}
