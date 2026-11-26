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

macro_rules! text_method {
    ($name:literal, $descriptor:literal, ($($arg:ident: $ty:ty),*) -> $result:ty) => {{
        async fn forward(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextBox>, $($arg: $ty),*) -> JvmResult<$result> {
            tracing::debug!(concat!("javax.microedition.lcdui.TextBox::", $name, "{:?}"), (&this, $(&$arg,)*));
            let field: ClassInstanceRef<TextField> = jvm.get_field(&this, TEXT_BOX, "editor", FIELD_TYPE).await?;
            jvm.invoke_virtual(&field, FIELD, $name, $descriptor, ($($arg,)*)).await
        }
        JavaMethodProto::new($name, $descriptor, forward, MethodAccessFlags::PUBLIC)
    }};
}

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
                text_method!("getString", "()Ljava/lang/String;", () -> ClassInstanceRef<String>),
                text_method!("setString", "(Ljava/lang/String;)V", (text: ClassInstanceRef<String>) -> ()),
                text_method!("getChars", "([C)I", (data: ClassInstanceRef<Array<JavaChar>>) -> i32),
                text_method!("setChars", "([CII)V", (data: ClassInstanceRef<Array<JavaChar>>, offset: i32, length: i32) -> ()),
                text_method!("insert", "(Ljava/lang/String;I)V", (text: ClassInstanceRef<String>, position: i32) -> ()),
                text_method!("insert", "([CIII)V", (data: ClassInstanceRef<Array<JavaChar>>, offset: i32, length: i32, position: i32) -> ()),
                text_method!("delete", "(II)V", (offset: i32, length: i32) -> ()),
                text_method!("getMaxSize", "()I", () -> i32),
                text_method!("setMaxSize", "(I)I", (max_size: i32) -> i32),
                text_method!("size", "()I", () -> i32),
                text_method!("getCaretPosition", "()I", () -> i32),
                text_method!("getConstraints", "()I", () -> i32),
                text_method!("setConstraints", "(I)V", (constraints: i32) -> ()),
                text_method!("setInitialInputMode", "(Ljava/lang/String;)V", (subset: ClassInstanceRef<String>) -> ()),
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
