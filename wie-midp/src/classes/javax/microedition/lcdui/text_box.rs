use alloc::vec;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Font, Graphics, Item, TextField},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

const TEXT_BOX: &str = "javax/microedition/lcdui/TextBox";
const FIELD: &str = "javax/microedition/lcdui/TextField";
const FIELD_TYPE: &str = "Ljavax/microedition/lcdui/TextField;";
const DISPLAYABLE: &str = "javax/microedition/lcdui/Displayable";
const SCREEN: &str = "javax/microedition/lcdui/Screen";

// class javax.microedition.lcdui.TextBox
pub struct TextBox;

impl TextBox {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: TEXT_BOX,
            parent_class: Some(SCREEN),
            interfaces: vec![],
            methods: vec![
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
                JavaMethodProto::new(
                    "itemInvalidated",
                    "(Ljavax/microedition/lcdui/Item;Z)V",
                    Self::item_invalidated,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::handle_paint_event,
                    MethodAccessFlags::empty(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("editor", FIELD_TYPE, FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("scrollY", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::<init>({this:?}, {title:?}, {text:?}, {max_size}, {constraints})");

        let _: () = jvm.invoke_special(&this, SCREEN, "<init>", "()V", ()).await?;
        let field: ClassInstanceRef<TextField> = jvm
            .new_class(FIELD, "(Ljava/lang/String;Ljava/lang/String;II)V", (None, text, max_size, constraints))
            .await?
            .into();
        let _: () = jvm
            .invoke_virtual(
                &field,
                "javax/microedition/lcdui/Item",
                "setOwner",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (this.clone(),),
            )
            .await?;
        jvm.put_field(&mut this, TEXT_BOX, "editor", FIELD_TYPE, field).await?;
        jvm.invoke_special(&this, DISPLAYABLE, "setTitle", "(Ljava/lang/String;)V", (title,))
            .await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getString({this:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "getString", "()Ljava/lang/String;", ()).await
    }

    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setString({this:?}, {text:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "setString", "(Ljava/lang/String;)V", (text,)).await
    }

    async fn get_chars(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<JavaChar>>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getChars({this:?}, {data:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "getChars", "([C)I", (data,)).await
    }

    async fn set_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setChars({this:?}, {data:?}, {offset}, {length})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "setChars", "([CII)V", (data, offset, length)).await
    }

    async fn insert_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::insert({this:?}, {text:?}, {position})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "insert", "(Ljava/lang/String;I)V", (text, position))
            .await
    }

    async fn insert_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::insert({this:?}, {data:?}, {offset}, {length}, {position})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "insert", "([CIII)V", (data, offset, length, position))
            .await
    }

    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, offset: i32, length: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::delete({this:?}, {offset}, {length})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "delete", "(II)V", (offset, length)).await
    }

    async fn get_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getMaxSize({this:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "getMaxSize", "()I", ()).await
    }

    async fn set_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setMaxSize({this:?}, {max_size})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "setMaxSize", "(I)I", (max_size,)).await
    }

    async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::size({this:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "size", "()I", ()).await
    }

    async fn get_caret_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getCaretPosition({this:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "getCaretPosition", "()I", ()).await
    }

    async fn get_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getConstraints({this:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "getConstraints", "()I", ()).await
    }

    async fn set_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setConstraints({this:?}, {constraints})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "setConstraints", "(I)V", (constraints,)).await
    }

    async fn set_initial_input_mode(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        subset: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setInitialInputMode({this:?}, {subset:?})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        jvm.invoke_virtual(&field, FIELD, "setInitialInputMode", "(Ljava/lang/String;)V", (subset,))
            .await
    }

    async fn item_invalidated(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        item: ClassInstanceRef<Item>,
        layout_changed: bool,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::itemInvalidated({this:?}, {item:?}, {layout_changed})");
        jvm.invoke_virtual(&this, DISPLAYABLE, "requestRepaint", "()V", ()).await
    }

    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::handleKeyEvent({this:?}, {event_type}, {key})");
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        if event_type == KeyboardEventType::KeyPressed as i32 && key == MIDPKeyCode::FIRE as i32 {
            let _: i32 = jvm.invoke_virtual(&field, FIELD, "handleItemKey", "(I)I", (key,)).await?;
            return Ok(());
        }
        if event_type != KeyboardEventType::KeyPressed as i32 && event_type != KeyboardEventType::KeyRepeated as i32 {
            return Ok(());
        }
        if key != MIDPKeyCode::UP as i32 && key != MIDPKeyCode::DOWN as i32 {
            return Ok(());
        }
        let width: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getHeight", "()I", ()).await?;
        let content: i32 = jvm.invoke_virtual(&field, FIELD, "preferredContentHeight", "(I)I", (width,)).await?;
        let scroll: i32 = jvm.get_field(&this, TEXT_BOX, "scrollY", "I").await?;
        let delta = if key == MIDPKeyCode::UP as i32 { -Font::HEIGHT } else { Font::HEIGHT };
        let scroll = (scroll + delta).clamp(0, (content - height).max(0));
        jvm.put_field(&mut this, TEXT_BOX, "scrollY", "I", scroll).await?;
        jvm.invoke_virtual(&this, DISPLAYABLE, "requestRepaint", "()V", ()).await
    }

    async fn handle_paint_event(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::handlePaintEvent({this:?}, {graphics:?})");
        let _: () = jvm
            .invoke_special(
                &this,
                SCREEN,
                "handlePaintEvent",
                "(Ljavax/microedition/lcdui/Graphics;)V",
                (graphics.clone(),),
            )
            .await?;
        let width: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getHeight", "()I", ()).await?;
        let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
        let content: i32 = jvm.invoke_virtual(&field, FIELD, "preferredContentHeight", "(I)I", (width,)).await?;
        let scroll: i32 = jvm.get_field(&this, TEXT_BOX, "scrollY", "I").await?;
        let scroll = scroll.min((content - height).max(0));
        jvm.put_field(&mut this, TEXT_BOX, "scrollY", "I", scroll).await?;
        jvm.invoke_virtual(
            &field,
            FIELD,
            "paintContent",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, 0, -scroll, width, height.max(content), true),
        )
        .await
    }
}
