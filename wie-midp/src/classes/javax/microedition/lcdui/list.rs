use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{ChoiceGroup, Command, Font, Graphics, Image, Item},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

const LIST: &str = "javax/microedition/lcdui/List";
const GROUP: &str = "javax/microedition/lcdui/ChoiceGroup";
const GROUP_FIELD: &str = "Ljavax/microedition/lcdui/ChoiceGroup;";
const COMMAND_FIELD: &str = "Ljavax/microedition/lcdui/Command;";
const DISPLAYABLE: &str = "javax/microedition/lcdui/Displayable";
const SCREEN: &str = "javax/microedition/lcdui/Screen";
const WITH_ELEMENTS: &str = "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V";

macro_rules! choice_method {
    ($name:literal, $descriptor:literal, ($($arg:ident: $ty:ty),*) -> $result:ty) => {{
        async fn forward(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<List>, $($arg: $ty),*) -> JvmResult<$result> {
            tracing::debug!(concat!("javax.microedition.lcdui.List::", $name, "{:?}"), (&this, $(&$arg,)*));
            let group: ClassInstanceRef<ChoiceGroup> = jvm.get_field(&this, LIST, "choices", GROUP_FIELD).await?;
            jvm.invoke_virtual(&group, GROUP, $name, $descriptor, ($($arg,)*)).await
        }
        JavaMethodProto::new($name, $descriptor, forward, MethodAccessFlags::PUBLIC)
    }};
}

// class javax.microedition.lcdui.List
pub struct List;

impl List {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: LIST,
            parent_class: Some(SCREEN),
            interfaces: vec!["javax/microedition/lcdui/Choice"],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", WITH_ELEMENTS, Self::init_with_elements, MethodAccessFlags::PUBLIC),
                choice_method!("size", "()I", () -> i32),
                choice_method!("getString", "(I)Ljava/lang/String;", (index: i32) -> ClassInstanceRef<String>),
                choice_method!("getImage", "(I)Ljavax/microedition/lcdui/Image;", (index: i32) -> ClassInstanceRef<Image>),
                choice_method!("append", "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I", (text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> i32),
                choice_method!("insert", "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V", (index: i32, text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> ()),
                choice_method!("delete", "(I)V", (index: i32) -> ()),
                choice_method!("deleteAll", "()V", () -> ()),
                choice_method!("set", "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V", (index: i32, text: ClassInstanceRef<String>, image: ClassInstanceRef<Image>) -> ()),
                choice_method!("isSelected", "(I)Z", (index: i32) -> bool),
                choice_method!("getSelectedIndex", "()I", () -> i32),
                choice_method!("getSelectedFlags", "([Z)I", (flags: ClassInstanceRef<Array<bool>>) -> i32),
                choice_method!("setSelectedIndex", "(IZ)V", (index: i32, selected: bool) -> ()),
                choice_method!("setSelectedFlags", "([Z)V", (flags: ClassInstanceRef<Array<bool>>) -> ()),
                choice_method!("setFitPolicy", "(I)V", (policy: i32) -> ()),
                choice_method!("getFitPolicy", "()I", () -> i32),
                choice_method!("setFont", "(ILjavax/microedition/lcdui/Font;)V", (index: i32, font: ClassInstanceRef<Font>) -> ()),
                choice_method!("getFont", "(I)Ljavax/microedition/lcdui/Font;", (index: i32) -> ClassInstanceRef<Font>),
                JavaMethodProto::new(
                    "setSelectCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::set_select_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "removeCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::remove_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "dispatchCommand",
                    "(Ljavax/microedition/lcdui/Command;)Z",
                    Self::dispatch_command,
                    MethodAccessFlags::empty(),
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
                JavaFieldProto::new(
                    "SELECT_COMMAND",
                    COMMAND_FIELD,
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("selectCommand", COMMAND_FIELD, FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("choices", GROUP_FIELD, FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("scrollY", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<clinit>");
        let label = JavaLangString::from_rust_string(jvm, "").await?;
        let command = jvm
            .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 1, 0))
            .await?;
        jvm.put_static_field(LIST, "SELECT_COMMAND", COMMAND_FIELD, command).await
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>, list_type: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type})");
        let strings = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_special(&this, LIST, "<init>", WITH_ELEMENTS, (title, list_type, strings, None))
            .await
    }

    async fn init_with_elements(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        list_type: i32,
        strings: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
        images: ClassInstanceRef<Array<ClassInstanceRef<Image>>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type}, {strings:?}, {images:?})");
        if !(1..=3).contains(&list_type) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid List type").await);
        }
        let _: () = jvm.invoke_special(&this, SCREEN, "<init>", "()V", ()).await?;
        let group = jvm
            .new_class(
                GROUP,
                "(Ljavax/microedition/lcdui/List;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                (this.clone(), list_type, strings, images),
            )
            .await?;
        jvm.put_field(&mut this, LIST, "choices", GROUP_FIELD, group).await?;
        if list_type == 3 {
            let select: ClassInstanceRef<Command> = jvm.get_static_field(LIST, "SELECT_COMMAND", COMMAND_FIELD).await?;
            jvm.put_field(&mut this, LIST, "selectCommand", COMMAND_FIELD, select).await?;
        }
        jvm.invoke_special(&this, DISPLAYABLE, "setTitle", "(Ljava/lang/String;)V", (title,))
            .await
    }

    async fn set_select_command(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setSelectCommand({this:?}, {command:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm.get_field(&this, LIST, "choices", GROUP_FIELD).await?;
        if jvm.get_field::<i32>(&group, GROUP, "choiceType", "I").await? == 3 {
            let default: ClassInstanceRef<Command> = jvm.get_static_field(LIST, "SELECT_COMMAND", COMMAND_FIELD).await?;
            if !command.is_null() && command.identity() != default.identity() {
                let _: () = jvm
                    .invoke_special(
                        &this,
                        DISPLAYABLE,
                        "addCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command.clone(),),
                    )
                    .await?;
            }
            jvm.put_field(&mut this, LIST, "selectCommand", COMMAND_FIELD, command).await?;
        }
        Ok(())
    }

    async fn remove_command(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::removeCommand({this:?}, {command:?})");
        let select: ClassInstanceRef<Command> = jvm.get_field(&this, LIST, "selectCommand", COMMAND_FIELD).await?;
        if !command.is_null() && !select.is_null() && command.identity() == select.identity() {
            jvm.put_field(&mut this, LIST, "selectCommand", COMMAND_FIELD, None).await?;
        }
        jvm.invoke_special(&this, DISPLAYABLE, "removeCommand", "(Ljavax/microedition/lcdui/Command;)V", (command,))
            .await
    }

    async fn dispatch_command(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.List::dispatchCommand({this:?}, {command:?})");
        let select: ClassInstanceRef<Command> = jvm.get_field(&this, LIST, "selectCommand", COMMAND_FIELD).await?;
        if !select.is_null() && command.identity() == select.identity() && jvm.invoke_special::<_, i32>(&this, LIST, "size", "()I", ()).await? == 0 {
            return Ok(false);
        }
        jvm.invoke_special(&this, DISPLAYABLE, "dispatchCommand", "(Ljavax/microedition/lcdui/Command;)Z", (command,))
            .await
    }

    async fn item_invalidated(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        item: ClassInstanceRef<Item>,
        layout_changed: bool,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::itemInvalidated({this:?}, {item:?}, {layout_changed})");
        jvm.invoke_virtual(&this, DISPLAYABLE, "requestRepaint", "()V", ()).await
    }

    // Both input and paint use the current row bounds, since the application can edit
    // the choices or change the available screen height between key events.
    async fn scroll_to_focus(
        jvm: &Jvm,
        this: &mut ClassInstanceRef<Self>,
        group: &ClassInstanceRef<ChoiceGroup>,
        width: i32,
        height: i32,
        page_delta: i32,
    ) -> JvmResult<(i32, bool)> {
        let bounds: ClassInstanceRef<Array<i32>> = jvm.invoke_virtual(group, GROUP, "getFocusContentBounds", "(I)[I", (width,)).await?;
        if bounds.is_null() {
            return Ok((0, false));
        }
        let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
        let top = bounds[0];
        let last_page = bounds[1] - height;
        let previous: i32 = jvm.get_field(this, LIST, "scrollY", "I").await?;
        let (scroll, paged) = if last_page > top {
            // Page through an oversized row before moving focus to its neighbor.
            let current = previous.clamp(top, last_page);
            let next = current.saturating_add(page_delta).clamp(top, last_page);
            (next, next != current)
        } else {
            let content_height: i32 = jvm.invoke_virtual(group, GROUP, "preferredContentHeight", "(I)I", (width,)).await?;
            (previous.clamp(last_page, top).clamp(0, (content_height - height).max(0)), false)
        };
        jvm.put_field(this, LIST, "scrollY", "I", scroll).await?;
        Ok((scroll, paged))
    }

    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::handleKeyEvent({this:?}, {event_type}, {code})");
        let vertical = code == MIDPKeyCode::UP as i32 || code == MIDPKeyCode::DOWN as i32;
        if event_type != KeyboardEventType::KeyPressed as i32 && !(vertical && event_type == KeyboardEventType::KeyRepeated as i32) {
            return Ok(());
        }
        let group: ClassInstanceRef<ChoiceGroup> = jvm.get_field(&this, LIST, "choices", GROUP_FIELD).await?;
        if vertical {
            let width: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getWidth", "()I", ()).await?;
            let height: i32 = jvm.invoke_special(&this, DISPLAYABLE, "getHeight", "()I", ()).await?;
            let delta = if code == MIDPKeyCode::UP as i32 { -height } else { height };
            let (_, paged) = Self::scroll_to_focus(jvm, &mut this, &group, width, height, delta).await?;
            if paged {
                return jvm.invoke_virtual(&this, DISPLAYABLE, "requestRepaint", "()V", ()).await;
            }
        }
        let _: i32 = jvm.invoke_virtual(&group, GROUP, "handleItemKey", "(I)I", (code,)).await?;
        if code == MIDPKeyCode::FIRE as i32 {
            let command: ClassInstanceRef<Command> = jvm.get_field(&this, LIST, "selectCommand", COMMAND_FIELD).await?;
            if !command.is_null() {
                let _: bool = jvm
                    .invoke_virtual(&this, LIST, "dispatchCommand", "(Ljavax/microedition/lcdui/Command;)Z", (command,))
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_paint_event(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::handlePaintEvent({this:?}, {graphics:?})");
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
        let group: ClassInstanceRef<ChoiceGroup> = jvm.get_field(&this, LIST, "choices", GROUP_FIELD).await?;
        let (scroll, _) = Self::scroll_to_focus(jvm, &mut this, &group, width, height, 0).await?;
        jvm.invoke_virtual(
            &group,
            GROUP,
            "paintContent",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, 0, -scroll, width, height, true),
        )
        .await
    }
}
