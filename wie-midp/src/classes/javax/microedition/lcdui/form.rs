use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::{
        lcdui::{Command, Display, Displayable, Graphics, Image, ImageItem, Item, ItemStateListener, StringItem},
        midlet::MIDlet,
    },
    net::wie::{ItemStateEvent, KeyboardEventType, MIDPKeyCode},
};

struct ItemRect {
    item: ClassInstanceRef<Item>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

// class javax.microedition.lcdui.Form
pub struct Form;

impl Form {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Form",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;[Ljavax/microedition/lcdui/Item;)V",
                    Self::init_with_items,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("append", "(Ljavax/microedition/lcdui/Item;)I", Self::append, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("append", "(Ljava/lang/String;)I", Self::append_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "append",
                    "(Ljavax/microedition/lcdui/Image;)I",
                    Self::append_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("insert", "(ILjavax/microedition/lcdui/Item;)V", Self::insert, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("set", "(ILjavax/microedition/lcdui/Item;)V", Self::set, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("get", "(I)Ljavax/microedition/lcdui/Item;", Self::get, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("delete", "(I)V", Self::delete, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deleteAll", "()V", Self::delete_all, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setItemStateListener",
                    "(Ljavax/microedition/lcdui/ItemStateListener;)V",
                    Self::set_item_state_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCommandCount", "()I", Self::get_command_count, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "getCommandAt",
                    "(I)Ljavax/microedition/lcdui/Command;",
                    Self::get_command_at,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("dispatchCommandAt", "(I)V", Self::dispatch_command_at, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "itemInvalidated",
                    "(Ljavax/microedition/lcdui/Item;Z)V",
                    Self::item_invalidated,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "itemStateChanged",
                    "(Ljavax/microedition/lcdui/Item;)V",
                    Self::item_state_changed,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "dispatchItemStateChanged",
                    "(Ljavax/microedition/lcdui/Item;)V",
                    Self::dispatch_item_state_changed,
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
                JavaFieldProto::new("items", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "itemStateListener",
                    "Ljavax/microedition/lcdui/ItemStateListener;",
                    FieldAccessFlags::PRIVATE,
                ),
                JavaFieldProto::new("focusIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("scrollY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("contentScrolling", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Form::<init>({this:?}, {title:?})");

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Form",
            "<init>",
            "(Ljava/lang/String;[Ljavax/microedition/lcdui/Item;)V",
            (title, None),
        )
        .await
    }

    async fn init_with_items(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        initial_items: ClassInstanceRef<Array<ClassInstanceRef<Item>>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Form::<init>({this:?}, {title:?}, {initial_items:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;
        let items: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_field(&mut this, "items", "Ljava/util/Vector;", items.clone()).await?;
        jvm.put_field(&mut this, "focusIndex", "I", -1).await?;
        jvm.put_field(&mut this, "scrollY", "I", 0).await?;
        jvm.put_field(&mut this, "contentScrolling", "Z", false).await?;
        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "setTitle",
                "(Ljava/lang/String;)V",
                (title,),
            )
            .await?;

        if initial_items.is_null() {
            return Ok(());
        }

        let item_count = jvm.array_length(&initial_items).await?;
        let initial_items: Vec<ClassInstanceRef<Item>> = jvm.load_array(&initial_items, 0, item_count).await?;
        for (index, item) in initial_items.iter().enumerate() {
            if item.is_null() {
                return Err(jvm.exception("java/lang/NullPointerException", "Form item is null").await);
            }
            let owner: ClassInstanceRef<Displayable> = jvm
                .invoke_virtual(
                    item,
                    "javax/microedition/lcdui/Item",
                    "getOwner",
                    "()Ljavax/microedition/lcdui/Displayable;",
                    (),
                )
                .await?;
            if !owner.is_null() || initial_items[..index].iter().any(|previous| previous.identity() == item.identity()) {
                return Err(jvm.exception("java/lang/IllegalStateException", "Form item already has an owner").await);
            }
        }

        for item in &initial_items {
            jvm.invoke_virtual::<_, ()>(
                item,
                "javax/microedition/lcdui/Item",
                "setOwner",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (this.clone(),),
            )
            .await?;
            let _: () = jvm
                .invoke_virtual(&items, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (item.clone(),))
                .await?;
        }

        Self::normalize_layout_state(jvm, context, &mut this, -1).await?;
        Ok(())
    }

    async fn checked_item(jvm: &Jvm, this: &ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Item>> {
        let items: ClassInstanceRef<Vector> = jvm.get_field(this, "items", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Err(jvm
                .exception("java/lang/IndexOutOfBoundsException", "Form item index is out of bounds")
                .await);
        }
        jvm.invoke_virtual(&items, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    async fn load_items(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<ClassInstanceRef<Item>>> {
        let items: ClassInstanceRef<Vector> = jvm.get_field(this, "items", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        let mut result = Vec::with_capacity(size as usize);
        for index in 0..size {
            result.push(
                jvm.invoke_virtual(&items, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                    .await?,
            );
        }
        Ok(result)
    }

    async fn layout_items(jvm: &Jvm, items: &[ClassInstanceRef<Item>], width: i32) -> JvmResult<Vec<ItemRect>> {
        let width = width.max(0);
        let mut rows = Vec::with_capacity(items.len());
        let mut y = 0i32;
        let mut alignment = 1;
        for item in items {
            let layout: i32 = jvm.invoke_virtual(item, "javax/microedition/lcdui/Item", "getLayout", "()I", ()).await?;
            if layout & 3 != 0 {
                alignment = layout & 3;
            }
            let item_width: i32 = jvm
                .invoke_virtual(item, "javax/microedition/lcdui/Item", "measureWidth", "(I)I", (width,))
                .await?;
            let x = match alignment {
                2 => width - item_width,
                3 => (width - item_width) / 2,
                _ => 0,
            };
            let height: i32 = jvm
                .invoke_virtual(item, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (item_width,))
                .await?;
            let height = height.max(1);
            rows.push(ItemRect {
                item: item.clone(),
                x,
                y,
                width: item_width,
                height,
            });
            y = y.saturating_add(height);
        }
        Ok(rows)
    }

    async fn update_scroll(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: &mut ClassInstanceRef<Self>,
        rows: &[ItemRect],
        viewport_height: i32,
    ) -> JvmResult<()> {
        let viewport_height = viewport_height.max(0);
        let content_height = rows.last().map(|row| row.y.saturating_add(row.height)).unwrap_or(0);
        let maximum_scroll = content_height.saturating_sub(viewport_height).max(0);
        let mut scroll: i32 = jvm.get_field(this, "scrollY", "I").await?;
        scroll = scroll.clamp(0, maximum_scroll);

        let focus_index: i32 = jvm.get_field(this, "focusIndex", "I").await?;
        let content_scrolling: bool = jvm.get_field(this, "contentScrolling", "Z").await?;
        if !content_scrolling
            && focus_index >= 0
            && let Some(row) = rows.get(focus_index as usize)
        {
            let (top, bottom) = if row.height > viewport_height {
                let bounds: ClassInstanceRef<Array<i32>> = jvm
                    .invoke_virtual(
                        &row.item,
                        "javax/microedition/lcdui/Item",
                        "getFocusBounds",
                        "(II)[I",
                        (row.width, row.height),
                    )
                    .await?;
                let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
                (bounds[0], bounds[1])
            } else {
                (0, row.height)
            };
            let top = row.y.saturating_add(top);
            let bottom = row.y.saturating_add(bottom);
            if bottom - top > viewport_height || top < scroll {
                scroll = top;
            } else if bottom > scroll.saturating_add(viewport_height) {
                scroll = bottom.saturating_sub(viewport_height);
            }
            scroll = scroll.clamp(0, maximum_scroll);
        }

        jvm.put_field(this, "scrollY", "I", scroll).await
    }

    async fn normalize_layout_state(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: &mut ClassInstanceRef<Self>,
        preferred_focus: i32,
    ) -> JvmResult<()> {
        jvm.put_field(this, "contentScrolling", "Z", false).await?;
        let items = Self::load_items(jvm, this).await?;
        if items.is_empty() {
            jvm.put_field(this, "focusIndex", "I", -1).await?;
            return jvm.put_field(this, "scrollY", "I", 0).await;
        }

        let mut focus_index = -1;
        if preferred_focus < 0 {
            for (index, item) in items.iter().enumerate() {
                if jvm
                    .invoke_virtual::<_, bool>(item, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ())
                    .await?
                {
                    focus_index = index as i32;
                    break;
                }
            }
        } else {
            let preferred_focus = preferred_focus.min(items.len() as i32 - 1) as usize;
            for (index, item) in items.iter().enumerate().skip(preferred_focus) {
                if jvm
                    .invoke_virtual::<_, bool>(item, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ())
                    .await?
                {
                    focus_index = index as i32;
                    break;
                }
            }
            if focus_index < 0 {
                for index in (0..preferred_focus).rev() {
                    if jvm
                        .invoke_virtual::<_, bool>(&items[index], "javax/microedition/lcdui/Item", "isFocusable", "()Z", ())
                        .await?
                    {
                        focus_index = index as i32;
                        break;
                    }
                }
            }
        }

        jvm.put_field(this, "focusIndex", "I", focus_index).await?;
        let width: i32 = jvm
            .invoke_special(this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_special(this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let rows = Self::layout_items(jvm, &items, width).await?;
        Self::update_scroll(jvm, context, this, &rows, height).await
    }

    async fn focused_item(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Item>> {
        let focus_index: i32 = jvm.get_field(this, "focusIndex", "I").await?;
        if focus_index < 0 {
            return Ok(None.into());
        }
        Self::checked_item(jvm, this, focus_index).await
    }

    async fn append(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, item: ClassInstanceRef<Item>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Form::append({this:?}, {item:?})");

        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        let index: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Form",
                "insert",
                "(ILjavax/microedition/lcdui/Item;)V",
                (index, item),
            )
            .await?;
        Ok(index)
    }

    async fn append_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Form::append({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Form string is null").await);
        }
        let item: ClassInstanceRef<StringItem> = jvm
            .new_class(
                "javax/microedition/lcdui/StringItem",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                (None, text),
            )
            .await?
            .into();
        jvm.invoke_virtual(
            &this,
            "javax/microedition/lcdui/Form",
            "append",
            "(Ljavax/microedition/lcdui/Item;)I",
            (item,),
        )
        .await
    }

    async fn append_image(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Form::append({this:?}, {image:?})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Form image is null").await);
        }
        let item: ClassInstanceRef<ImageItem> = jvm
            .new_class(
                "javax/microedition/lcdui/ImageItem",
                "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;)V",
                (None, image, 0, None),
            )
            .await?
            .into();
        jvm.invoke_virtual(
            &this,
            "javax/microedition/lcdui/Form",
            "append",
            "(Ljavax/microedition/lcdui/Item;)I",
            (item,),
        )
        .await
    }

    async fn insert(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index > size {
            return Err(jvm
                .exception("java/lang/IndexOutOfBoundsException", "Form insert index is out of bounds")
                .await);
        }
        if item.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Form item is null").await);
        }
        let owner: ClassInstanceRef<Displayable> = jvm
            .invoke_virtual(
                &item,
                "javax/microedition/lcdui/Item",
                "getOwner",
                "()Ljavax/microedition/lcdui/Displayable;",
                (),
            )
            .await?;
        if !owner.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Form item already has an owner").await);
        }

        jvm.invoke_virtual::<_, ()>(
            &item,
            "javax/microedition/lcdui/Item",
            "setOwner",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (this.clone(),),
        )
        .await?;
        let _: () = jvm
            .invoke_virtual(&items, "java/util/Vector", "insertElementAt", "(Ljava/lang/Object;I)V", (item, index))
            .await?;
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        let preferred_focus = if focus_index >= index { focus_index + 1 } else { focus_index };
        Self::normalize_layout_state(jvm, context, &mut this, preferred_focus).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn set(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        index: i32,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let old_item = Self::checked_item(jvm, &this, index).await?;
        if item.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Form item is null").await);
        }
        let owner: ClassInstanceRef<Displayable> = jvm
            .invoke_virtual(
                &item,
                "javax/microedition/lcdui/Item",
                "getOwner",
                "()Ljavax/microedition/lcdui/Displayable;",
                (),
            )
            .await?;
        if !owner.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Form item already has an owner").await);
        }

        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual::<_, ()>(
            &old_item,
            "javax/microedition/lcdui/Item",
            "setOwner",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (None,),
        )
        .await?;
        jvm.invoke_virtual::<_, ()>(
            &item,
            "javax/microedition/lcdui/Item",
            "setOwner",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (this.clone(),),
        )
        .await?;
        let _: () = jvm
            .invoke_virtual(&items, "java/util/Vector", "setElementAt", "(Ljava/lang/Object;I)V", (item, index))
            .await?;
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        Self::normalize_layout_state(jvm, context, &mut this, focus_index).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn get(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Item>> {
        Self::checked_item(jvm, &this, index).await
    }

    async fn delete(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        let item = Self::checked_item(jvm, &this, index).await?;
        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        let _: ClassInstanceRef<Item> = jvm
            .invoke_virtual(&items, "java/util/Vector", "remove", "(I)Ljava/lang/Object;", (index,))
            .await?;
        jvm.invoke_virtual::<_, ()>(
            &item,
            "javax/microedition/lcdui/Item",
            "setOwner",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (None,),
        )
        .await?;

        let size: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        let preferred_focus = if size == 0 {
            -1
        } else if focus_index > index {
            focus_index - 1
        } else {
            focus_index.min(size - 1)
        };
        Self::normalize_layout_state(jvm, context, &mut this, preferred_focus).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn delete_all(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        let size: i32 = jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await?;
        if size == 0 {
            return Ok(());
        }
        for index in 0..size {
            let item: ClassInstanceRef<Item> = jvm
                .invoke_virtual(&items, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                .await?;
            jvm.invoke_virtual::<_, ()>(
                &item,
                "javax/microedition/lcdui/Item",
                "setOwner",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (None,),
            )
            .await?;
        }
        let _: () = jvm.invoke_virtual(&items, "java/util/Vector", "removeAllElements", "()V", ()).await?;
        jvm.put_field(&mut this, "focusIndex", "I", -1).await?;
        jvm.put_field(&mut this, "scrollY", "I", 0).await?;
        jvm.put_field(&mut this, "contentScrolling", "Z", false).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let items: ClassInstanceRef<Vector> = jvm.get_field(&this, "items", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual(&items, "java/util/Vector", "size", "()I", ()).await
    }

    async fn set_item_state_listener(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemStateListener>,
    ) -> JvmResult<()> {
        jvm.put_field(&mut this, "itemStateListener", "Ljavax/microedition/lcdui/ItemStateListener;", listener)
            .await
    }

    async fn get_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await
    }

    async fn get_command_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let displayable_count: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        let item = Self::focused_item(jvm, &this).await?;
        if item.is_null() {
            return Ok(displayable_count);
        }

        let item_count: i32 = jvm
            .invoke_virtual(&item, "javax/microedition/lcdui/Item", "getCommandCount", "()I", ())
            .await?;
        Ok(item_count + displayable_count)
    }

    async fn get_command_at(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
    ) -> JvmResult<ClassInstanceRef<Command>> {
        let item = Self::focused_item(jvm, &this).await?;
        if !item.is_null() {
            let item_count: i32 = jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "getCommandCount", "()I", ())
                .await?;
            if index < item_count {
                return jvm
                    .invoke_virtual(
                        &item,
                        "javax/microedition/lcdui/Item",
                        "getCommandAt",
                        "(I)Ljavax/microedition/lcdui/Command;",
                        (index,),
                    )
                    .await;
            }
            return jvm
                .invoke_special(
                    &this,
                    "javax/microedition/lcdui/Displayable",
                    "getCommandAt",
                    "(I)Ljavax/microedition/lcdui/Command;",
                    (index - item_count,),
                )
                .await;
        }

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "getCommandAt",
            "(I)Ljavax/microedition/lcdui/Command;",
            (index,),
        )
        .await
    }

    async fn dispatch_command_at(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        let item = Self::focused_item(jvm, &this).await?;
        if !item.is_null() {
            let count: i32 = jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "getCommandCount", "()I", ())
                .await?;
            if index < count {
                let command: ClassInstanceRef<Command> = jvm
                    .invoke_virtual(
                        &item,
                        "javax/microedition/lcdui/Item",
                        "getCommandAt",
                        "(I)Ljavax/microedition/lcdui/Command;",
                        (index,),
                    )
                    .await?;
                return jvm
                    .invoke_virtual(
                        &item,
                        "javax/microedition/lcdui/Item",
                        "dispatchCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command,),
                    )
                    .await;
            }
            return jvm
                .invoke_special(
                    &this,
                    "javax/microedition/lcdui/Displayable",
                    "dispatchCommandAt",
                    "(I)V",
                    (index - count,),
                )
                .await;
        }

        jvm.invoke_special(&this, "javax/microedition/lcdui/Displayable", "dispatchCommandAt", "(I)V", (index,))
            .await
    }

    async fn item_invalidated(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
        _layout_changed: bool,
    ) -> JvmResult<()> {
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        Self::normalize_layout_state(jvm, context, &mut this, focus_index).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn item_state_changed(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let mut display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if display.is_null() {
            let midlet: ClassInstanceRef<MIDlet> = jvm
                .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
                .await?;
            display = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Display",
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    (midlet,),
                )
                .await?;
        }

        let event: ClassInstanceRef<ItemStateEvent> = jvm
            .new_class(
                "net/wie/ItemStateEvent",
                "(Ljavax/microedition/lcdui/Form;Ljavax/microedition/lcdui/Item;)V",
                (this, item),
            )
            .await?
            .into();
        jvm.invoke_virtual(
            &display,
            "javax/microedition/lcdui/Display",
            "callSerially",
            "(Ljava/lang/Runnable;)V",
            (event,),
        )
        .await
    }

    async fn dispatch_item_state_changed(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        let listener: ClassInstanceRef<ItemStateListener> = jvm
            .get_field(&this, "itemStateListener", "Ljavax/microedition/lcdui/ItemStateListener;")
            .await?;
        if listener.is_null() {
            return Ok(());
        }

        jvm.invoke_virtual(
            &listener,
            "javax/microedition/lcdui/ItemStateListener",
            "itemStateChanged",
            "(Ljavax/microedition/lcdui/Item;)V",
            (item,),
        )
        .await
    }

    async fn handle_key_event(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        let pressed = event_type == KeyboardEventType::KeyPressed as i32;
        let repeated = event_type == KeyboardEventType::KeyRepeated as i32;
        if !pressed && !repeated {
            return Ok(());
        }

        let vertical = code == MIDPKeyCode::UP as i32 || code == MIDPKeyCode::DOWN as i32;
        let mut scroll_context = (0, 0, 0);
        if vertical {
            let items = Self::load_items(jvm, &this).await?;
            let width: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
                .await?;
            let viewport_height: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                .await?;
            let viewport_height = viewport_height.max(0);
            let rows = Self::layout_items(jvm, &items, width).await?;
            let content_height = rows.last().map(|row| row.y.saturating_add(row.height)).unwrap_or(0);
            let maximum_scroll = content_height.saturating_sub(viewport_height).max(0);
            let mut scroll: i32 = jvm.get_field(&this, "scrollY", "I").await?;
            scroll = scroll.clamp(0, maximum_scroll);
            jvm.put_field(&mut this, "scrollY", "I", scroll).await?;

            if jvm.get_field::<bool>(&this, "contentScrolling", "Z").await? {
                let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
                let focus_visible = focus_index >= 0
                    && rows
                        .get(focus_index as usize)
                        .is_some_and(|row| row.y < scroll.saturating_add(viewport_height) && row.y.saturating_add(row.height) > scroll);
                if !focus_visible {
                    let step = viewport_height.max(1);
                    let new_scroll = if code == MIDPKeyCode::UP as i32 {
                        scroll.saturating_sub(step).max(0)
                    } else {
                        scroll.saturating_add(step).min(maximum_scroll)
                    };
                    if new_scroll != scroll {
                        jvm.put_field(&mut this, "scrollY", "I", new_scroll).await?;
                        let _: () = jvm
                            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                            .await?;
                    }
                    return Ok(());
                }
            }

            let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
            if focus_index >= 0
                && let Some(row) = rows.get(focus_index as usize)
                && row.height > viewport_height
            {
                let bounds: ClassInstanceRef<Array<i32>> = jvm
                    .invoke_virtual(
                        &row.item,
                        "javax/microedition/lcdui/Item",
                        "getFocusBounds",
                        "(II)[I",
                        (row.width, row.height),
                    )
                    .await?;
                let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
                let top = row.y.saturating_add(bounds[0]);
                let bottom = row.y.saturating_add(bounds[1]);
                // Read an oversized active span before moving focus, including inside an open popup.
                if bottom - top > viewport_height {
                    let new_scroll = if code == MIDPKeyCode::UP as i32 && scroll > top {
                        scroll.saturating_sub(viewport_height.max(1)).max(top)
                    } else if code == MIDPKeyCode::DOWN as i32 && scroll.saturating_add(viewport_height) < bottom {
                        scroll.saturating_add(viewport_height.max(1)).min(bottom - viewport_height)
                    } else {
                        scroll
                    }
                    .clamp(0, maximum_scroll);
                    if new_scroll != scroll {
                        jvm.put_field(&mut this, "scrollY", "I", new_scroll).await?;
                        jvm.put_field(&mut this, "contentScrolling", "Z", true).await?;
                        return jvm
                            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                            .await;
                    }
                }
            }
            scroll_context = (viewport_height, scroll, maximum_scroll);
        }

        let item = Self::focused_item(jvm, &this).await?;
        let directional = matches!(
            code,
            x if x == MIDPKeyCode::UP as i32
                || x == MIDPKeyCode::DOWN as i32
                || x == MIDPKeyCode::LEFT as i32
                || x == MIDPKeyCode::RIGHT as i32
        );
        if !item.is_null() && (pressed || directional) {
            let result: i32 = jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "handleItemKey", "(I)I", (code,))
                .await?;
            if result & Item::INPUT_CHANGED != 0 {
                let _: () = jvm
                    .invoke_virtual(
                        &this,
                        "javax/microedition/lcdui/Displayable",
                        "itemStateChanged",
                        "(Ljavax/microedition/lcdui/Item;)V",
                        (item.clone(),),
                    )
                    .await?;
            }
            if result & Item::INPUT_HANDLED != 0 {
                return Ok(());
            }
        }

        if pressed && code == MIDPKeyCode::FIRE as i32 && !item.is_null() {
            return jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "dispatchDefaultCommand", "()V", ())
                .await;
        }

        if code != MIDPKeyCode::UP as i32 && code != MIDPKeyCode::DOWN as i32 {
            return Ok(());
        }

        let items = Self::load_items(jvm, &this).await?;
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        let mut next_focus = None;
        if code == MIDPKeyCode::UP as i32 {
            for index in (0..focus_index.max(0) as usize).rev() {
                if jvm
                    .invoke_virtual::<_, bool>(&items[index], "javax/microedition/lcdui/Item", "isFocusable", "()Z", ())
                    .await?
                {
                    next_focus = Some(index);
                    break;
                }
            }
        } else {
            for (index, item) in items.iter().enumerate().skip(focus_index.saturating_add(1).max(0) as usize) {
                if jvm
                    .invoke_virtual::<_, bool>(item, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ())
                    .await?
                {
                    next_focus = Some(index);
                    break;
                }
            }
        }

        if let Some(index) = next_focus {
            Self::normalize_layout_state(jvm, context, &mut this, index as i32).await?;
            return jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
                .await;
        }

        let (viewport_height, scroll, maximum_scroll) = scroll_context;
        let step = viewport_height.max(1);
        let new_scroll = if code == MIDPKeyCode::UP as i32 {
            scroll.saturating_sub(step).max(0)
        } else {
            scroll.saturating_add(step).min(maximum_scroll)
        };
        if new_scroll != scroll {
            jvm.put_field(&mut this, "scrollY", "I", new_scroll).await?;
            jvm.put_field(&mut this, "contentScrolling", "Z", true).await?;
            return jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                .await;
        }

        Ok(())
    }

    async fn handle_paint_event(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/Screen",
                "handlePaintEvent",
                "(Ljavax/microedition/lcdui/Graphics;)V",
                (graphics.clone(),),
            )
            .await?;
        let width: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let items = Self::load_items(jvm, &this).await?;
        let rows = Self::layout_items(jvm, &items, width).await?;
        Self::update_scroll(jvm, context, &mut this, &rows, height).await?;

        let scroll: i32 = jvm.get_field(&this, "scrollY", "I").await?;
        let focus_index: i32 = jvm.get_field(&this, "focusIndex", "I").await?;
        let viewport_bottom = scroll.saturating_add(height.max(0));
        for (index, row) in rows.iter().enumerate() {
            if row.y.saturating_add(row.height) <= scroll || row.y >= viewport_bottom {
                continue;
            }
            let _: () = jvm
                .invoke_virtual(
                    &row.item,
                    "javax/microedition/lcdui/Item",
                    "paintItem",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    (
                        graphics.clone(),
                        row.x,
                        row.y - scroll,
                        row.width,
                        row.height,
                        index as i32 == focus_index,
                    ),
                )
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use test_utils::run_jvm_test;
    use wie_backend::{Event, KeyCode};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::lcdui::{
                ChoiceGroup, Command, CommandListener, Display, Displayable, Form, Gauge, Graphics, Image, Item, ItemCommandListener,
                ItemStateListener, StringItem,
            },
            javax::microedition::midlet::MIDlet,
            net::wie::{EventQueue, KeyboardEventType, MIDPKeyCode},
        },
        get_protos,
    };

    struct RecordingFormItem;
    struct RecordingFormItemStateListener;
    struct RecordingFormItemCommandListener;
    struct RecordingFormCommandListener;
    struct RecordingFormMidlet;
    struct BackendEventInjector;

    impl RecordingFormItem {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingFormItem",
                parent_class: Some("javax/microedition/lcdui/Item"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "(IIZ)V", Self::init, MethodAccessFlags::PUBLIC),
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
                ],
                fields: vec![
                    JavaFieldProto::new("contentHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("paintColor", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("focusable", "Z", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("paintCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastPaintY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastFocused", "Z", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            content_height: i32,
            paint_color: i32,
            focusable: bool,
        ) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
            jvm.put_field(&mut this, "contentHeight", "I", content_height).await?;
            jvm.put_field(&mut this, "paintColor", "I", paint_color).await?;
            jvm.put_field(&mut this, "focusable", "Z", focusable).await?;
            jvm.put_field(&mut this, "lastPaintY", "I", -1).await
        }

        async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            Ok(1)
        }

        async fn minimum_content_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            jvm.get_field(&this, "contentHeight", "I").await
        }

        async fn preferred_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            Ok(32)
        }

        async fn preferred_content_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, _width: i32) -> JvmResult<i32> {
            jvm.get_field(&this, "contentHeight", "I").await
        }

        #[allow(clippy::too_many_arguments)]
        async fn paint_content(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            graphics: ClassInstanceRef<Graphics>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            focused: bool,
        ) -> JvmResult<()> {
            let paint_count: i32 = jvm.get_field(&this, "paintCount", "I").await?;
            let color: i32 = jvm.get_field(&this, "paintColor", "I").await?;
            jvm.put_field(&mut this, "paintCount", "I", paint_count + 1).await?;
            jvm.put_field(&mut this, "lastPaintY", "I", y).await?;
            jvm.put_field(&mut this, "lastFocused", "Z", focused).await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
                .await?;
            jvm.invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await
        }

        async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
            if jvm.get_field::<bool>(&this, "focusable", "Z").await? {
                return Ok(true);
            }
            jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ()).await
        }
    }

    impl RecordingFormItemStateListener {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingFormItemStateListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/lcdui/ItemStateListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "itemStateChanged",
                        "(Ljavax/microedition/lcdui/Item;)V",
                        Self::item_state_changed,
                        MethodAccessFlags::PUBLIC,
                    ),
                ],
                fields: vec![
                    JavaFieldProto::new("count", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastItem", "Ljavax/microedition/lcdui/Item;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn item_state_changed(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            item: ClassInstanceRef<Item>,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "count", "I").await?;
            jvm.put_field(&mut this, "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastItem", "Ljavax/microedition/lcdui/Item;", item).await
        }
    }

    impl RecordingFormItemCommandListener {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingFormItemCommandListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/lcdui/ItemCommandListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "commandAction",
                        "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Item;)V",
                        Self::command_action,
                        MethodAccessFlags::PUBLIC,
                    ),
                ],
                fields: vec![
                    JavaFieldProto::new("count", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastCommand", "Ljavax/microedition/lcdui/Command;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastItem", "Ljavax/microedition/lcdui/Item;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn command_action(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            command: ClassInstanceRef<Command>,
            item: ClassInstanceRef<Item>,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "count", "I").await?;
            jvm.put_field(&mut this, "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastCommand", "Ljavax/microedition/lcdui/Command;", command)
                .await?;
            jvm.put_field(&mut this, "lastItem", "Ljavax/microedition/lcdui/Item;", item).await
        }
    }

    impl RecordingFormCommandListener {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingFormCommandListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/lcdui/CommandListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "commandAction",
                        "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                        Self::command_action,
                        MethodAccessFlags::PUBLIC,
                    ),
                ],
                fields: vec![
                    JavaFieldProto::new("count", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastCommand", "Ljavax/microedition/lcdui/Command;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastDisplayable", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn command_action(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            command: ClassInstanceRef<Command>,
            displayable: ClassInstanceRef<Displayable>,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "count", "I").await?;
            jvm.put_field(&mut this, "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastCommand", "Ljavax/microedition/lcdui/Command;", command)
                .await?;
            jvm.put_field(&mut this, "lastDisplayable", "Ljavax/microedition/lcdui/Displayable;", displayable)
                .await
        }
    }

    impl RecordingFormMidlet {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingFormMidlet",
                parent_class: Some("javax/microedition/midlet/MIDlet"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("startApp", "()V", Self::start_app, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await
        }

        async fn start_app(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<()> {
            Ok(())
        }
    }

    impl BackendEventInjector {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestBackendEventInjector",
                parent_class: Some("java/lang/Object"),
                interfaces: vec![],
                methods: vec![JavaMethodProto::new(
                    "enqueueOrderingInputs",
                    "()V",
                    Self::enqueue_ordering_inputs,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                )],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn enqueue_ordering_inputs(_jvm: &Jvm, context: &mut WieJvmContext) -> JvmResult<()> {
            let mut queue = context.system().event_queue();
            queue.push(Event::Keydown(KeyCode::LEFT));
            queue.push(Event::Keydown(KeyCode::RIGHT));
            queue.push(Event::Keydown(KeyCode::RIGHT));
            queue.push(Event::Keydown(KeyCode::DOWN));
            queue.push(Event::Keydown(KeyCode::RIGHT));
            queue.push(Event::Keydown(KeyCode::LEFT_SOFT_KEY));
            queue.push(Event::Redraw);
            Ok(())
        }
    }

    fn test_protos() -> Box<[Box<[WieJavaClassProto]>]> {
        Box::new([
            get_protos().into(),
            [
                RecordingFormItem::as_proto(),
                RecordingFormItemStateListener::as_proto(),
                RecordingFormItemCommandListener::as_proto(),
                RecordingFormCommandListener::as_proto(),
                RecordingFormMidlet::as_proto(),
                BackendEventInjector::as_proto(),
            ]
            .into(),
        ])
    }

    async fn recording_item(jvm: &Jvm, height: i32, color: i32, focusable: bool) -> JvmResult<ClassInstanceRef<RecordingFormItem>> {
        Ok(jvm
            .new_class("javax/microedition/lcdui/TestRecordingFormItem", "(IIZ)V", (height, color, focusable))
            .await?
            .into())
    }

    async fn form_with_items(jvm: &Jvm, items: Vec<ClassInstanceRef<Item>>) -> JvmResult<ClassInstanceRef<Form>> {
        let mut array: ClassInstanceRef<Array<ClassInstanceRef<Item>>> =
            jvm.instantiate_array("[Ljavax/microedition/lcdui/Item;", items.len()).await?.into();
        jvm.store_array(&mut array, 0, items).await?;
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Form",
                "(Ljava/lang/String;[Ljavax/microedition/lcdui/Item;)V",
                (None, array),
            )
            .await?
            .into())
    }

    async fn make_command(jvm: &Jvm, label: &str, command_type: i32, priority: i32) -> JvmResult<ClassInstanceRef<Command>> {
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Command",
                "(Ljava/lang/String;II)V",
                (JavaLangString::from_rust_string(jvm, label).await?, command_type, priority),
            )
            .await?
            .into())
    }

    async fn show_form(jvm: &Jvm, form: &ClassInstanceRef<Form>, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Display>> {
        let mut display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
        jvm.put_field(&mut display, "width", "I", width).await?;
        jvm.put_field(&mut display, "height", "I", height).await?;
        let _: () = jvm
            .invoke_virtual(
                &display,
                "javax/microedition/lcdui/Display",
                "setCurrent",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (form.clone(),),
            )
            .await?;
        Ok(display)
    }

    async fn send_key(jvm: &Jvm, display: &ClassInstanceRef<Display>, event_type: KeyboardEventType, key: MIDPKeyCode) -> JvmResult<()> {
        jvm.invoke_virtual(
            display,
            "javax/microedition/lcdui/Display",
            "handleKeyEvent",
            "(II)V",
            (event_type as i32, key as i32),
        )
        .await
    }

    #[test]
    fn form_rejects_duplicate_ownership_and_releases_removed_items() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let first = recording_item(&jvm, 8, 0x114411, false).await?;
            let second = recording_item(&jvm, 8, 0x225522, false).await?;
            let duplicate = form_with_items(&jvm, vec![JavaValue::from(first.clone()).into(), JavaValue::from(first.clone()).into()]).await;
            let Err(JavaError::JavaException(exception)) = duplicate else {
                panic!("Form accepted a duplicate Item");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));

            // A failed constructor must not acquire ownership of its earlier Items.
            let source = form_with_items(&jvm, vec![JavaValue::from(first.clone()).into()]).await?;
            let destination = form_with_items(&jvm, vec![]).await?;
            let invalid: JvmResult<i32> = jvm
                .invoke_virtual(
                    &destination,
                    "javax/microedition/lcdui/Form",
                    "append",
                    "(Ljavax/microedition/lcdui/Item;)I",
                    (first.clone(),),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = invalid else {
                panic!("Form accepted another Form's Item");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&destination, "javax/microedition/lcdui/Form", "size", "()I", ())
                    .await?,
                0
            );
            let retained: ClassInstanceRef<Item> = jvm
                .invoke_virtual(
                    &source,
                    "javax/microedition/lcdui/Form",
                    "get",
                    "(I)Ljavax/microedition/lcdui/Item;",
                    (0,),
                )
                .await?;
            assert_eq!(retained.identity(), first.identity());

            let _: () = jvm
                .invoke_virtual(
                    &source,
                    "javax/microedition/lcdui/Form",
                    "set",
                    "(ILjavax/microedition/lcdui/Item;)V",
                    (0, second.clone()),
                )
                .await?;
            let index: i32 = jvm
                .invoke_virtual(
                    &destination,
                    "javax/microedition/lcdui/Form",
                    "append",
                    "(Ljavax/microedition/lcdui/Item;)I",
                    (first.clone(),),
                )
                .await?;
            assert_eq!(index, 0);
            let _: () = jvm
                .invoke_virtual(&source, "javax/microedition/lcdui/Form", "delete", "(I)V", (0,))
                .await?;
            let index: i32 = jvm
                .invoke_virtual(
                    &destination,
                    "javax/microedition/lcdui/Form",
                    "append",
                    "(Ljavax/microedition/lcdui/Item;)I",
                    (second.clone(),),
                )
                .await?;
            assert_eq!(index, 1);
            let _: () = jvm
                .invoke_virtual(&destination, "javax/microedition/lcdui/Form", "deleteAll", "()V", ())
                .await?;
            let reused = form_with_items(&jvm, vec![JavaValue::from(first).into(), JavaValue::from(second).into()]).await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&reused, "javax/microedition/lcdui/Form", "size", "()I", ())
                    .await?,
                2
            );
            Ok(())
        })
    }

    #[test]
    fn form_normalizes_focus_across_construction_and_item_mutations() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let first = recording_item(&jvm, 8, 0x110000, false).await?;
            let focused = recording_item(&jvm, 8, 0x220000, true).await?;
            let candidate = recording_item(&jvm, 8, 0x330000, false).await?;
            let later = recording_item(&jvm, 8, 0x440000, true).await?;
            let form = form_with_items(
                &jvm,
                vec![
                    JavaValue::from(first).into(),
                    JavaValue::from(focused.clone()).into(),
                    JavaValue::from(candidate.clone()).into(),
                    JavaValue::from(later.clone()).into(),
                ],
            )
            .await?;
            assert_eq!(jvm.get_field::<i32>(&form, "focusIndex", "I").await?, 1);

            let inserted = recording_item(&jvm, 8, 0x550000, true).await?;
            let inserted_item: ClassInstanceRef<Item> = JavaValue::from(inserted.clone()).into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Form",
                    "insert",
                    "(ILjavax/microedition/lcdui/Item;)V",
                    (0, inserted_item),
                )
                .await?;
            assert_eq!(
                jvm.get_field::<i32>(&form, "focusIndex", "I").await?,
                2,
                "insertion must preserve the focused Item"
            );

            let _: () = jvm.invoke_virtual(&form, "javax/microedition/lcdui/Form", "delete", "(I)V", (2,)).await?;
            assert_eq!(
                jvm.get_field::<i32>(&form, "focusIndex", "I").await?,
                3,
                "deleting focus must select the next focusable Item"
            );

            let replacement = recording_item(&jvm, 8, 0x660000, false).await?;
            let replacement_item: ClassInstanceRef<Item> = JavaValue::from(replacement).into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Form",
                    "set",
                    "(ILjavax/microedition/lcdui/Item;)V",
                    (3, replacement_item),
                )
                .await?;
            assert_eq!(
                jvm.get_field::<i32>(&form, "focusIndex", "I").await?,
                0,
                "focus must fall back to the preceding focusable Item"
            );

            let _: () = jvm.invoke_virtual(&form, "javax/microedition/lcdui/Form", "delete", "(I)V", (0,)).await?;
            assert_eq!(jvm.get_field::<i32>(&form, "focusIndex", "I").await?, -1);

            let command = make_command(&jvm, "Focus", 8, 0).await?;
            let _: () = jvm
                .invoke_virtual(
                    &candidate,
                    "javax/microedition/lcdui/Item",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command.clone(),),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&form, "focusIndex", "I").await?, 1);

            let display = show_form(&jvm, &form, 60, 40).await?;
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
            assert!(jvm.get_field::<bool>(&candidate, "lastFocused", "Z").await?);

            let _: () = jvm
                .invoke_virtual(
                    &candidate,
                    "javax/microedition/lcdui/Item",
                    "removeCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command,),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&form, "focusIndex", "I").await?, -1);

            Ok(())
        })
    }

    #[test]
    fn form_paints_items_at_their_allocated_width_and_alignment() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let text = JavaLangString::from_rust_string(&jvm, "alpha beta gamma delta epsilon").await?;
            let item: ClassInstanceRef<StringItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/StringItem",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    (None, text),
                )
                .await?
                .into();
            let _: () = jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "setPreferredSize", "(II)V", (40, -1))
                .await?;
            let height: i32 = jvm
                .invoke_virtual(&item, "javax/microedition/lcdui/Item", "getPreferredHeight", "()I", ())
                .await?;
            let following = recording_item(&jvm, 4, 0x225522, false).await?;
            let form = form_with_items(
                &jvm,
                vec![JavaValue::from(item.clone()).into(), JavaValue::from(following.clone()).into()],
            )
            .await?;
            let display = show_form(&jvm, &form, 80, 240).await?;
            let mut graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "getScreenGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let expected: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (80, height),
                )
                .await?;
            let expected_graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &expected,
                    "javax/microedition/lcdui/Image",
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;

            for (layout, x) in [(0, 0), (1, 0), (2, 40), (3, 20), (3 | 0x800, 20), (2 | 0x400, 40)] {
                let _: () = jvm
                    .invoke_virtual(&item, "javax/microedition/lcdui/Item", "setLayout", "(I)V", (layout,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&expected_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xffffff,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &expected_graphics,
                        "javax/microedition/lcdui/Graphics",
                        "fillRect",
                        "(IIII)V",
                        (0, 0, 80, height),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &item,
                        "javax/microedition/lcdui/Item",
                        "paintItem",
                        "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                        (expected_graphics.clone(), x, 0, 40, height, false),
                    )
                    .await?;
                let actual = Graphics::image(&jvm, &mut graphics).await?;
                let actual = Image::image(&jvm, &actual).await?;
                let expected_pixels = Image::image(&jvm, &expected).await?;
                for y in 0..height {
                    for x in 0..80 {
                        let actual = actual.get_pixel(x, y);
                        let expected = expected_pixels.get_pixel(x, y);
                        assert_eq!(
                            (actual.r, actual.g, actual.b),
                            (expected.r, expected.g, expected.b),
                            "layout={layout}, x={x}, y={y}"
                        );
                    }
                }
                assert_eq!(jvm.get_field::<i32>(&following, "lastPaintY", "I").await?, height);
            }

            let _: () = jvm
                .invoke_virtual(&following, "javax/microedition/lcdui/Item", "setPreferredSize", "(II)V", (40, -1))
                .await?;
            for (layout, x, width) in [(0, 40, 40), (3, 20, 40), (0x800, 0, 80), (1 | 0x400, 0, 40)] {
                let _: () = jvm
                    .invoke_virtual(&following, "javax/microedition/lcdui/Item", "setLayout", "(I)V", (layout,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                let actual = Graphics::image(&jvm, &mut graphics).await?;
                let actual = Image::image(&jvm, &actual).await?;
                for column in 0..80 {
                    let color = actual.get_pixel(column, height);
                    let expected = if (x..x + width).contains(&column) {
                        (0x22, 0x55, 0x22)
                    } else {
                        (0xff, 0xff, 0xff)
                    };
                    assert_eq!((color.r, color.g, color.b), expected, "layout={layout}, x={column}");
                }
            }
            Ok(())
        })
    }

    #[test]
    fn form_reveals_oversized_choice_groups_and_scrolls_the_final_element() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            for choice_type in [1, 4] {
                let choice: ClassInstanceRef<ChoiceGroup> = jvm
                    .new_class(
                        "javax/microedition/lcdui/ChoiceGroup",
                        "(Ljava/lang/String;I)V",
                        (JavaLangString::from_rust_string(&jvm, "Options\nChoices").await?, choice_type),
                    )
                    .await?
                    .into();
                for index in 0..30 {
                    let text = if index == 10 { "Choice\ncontinued" } else { "Choice" };
                    let _: i32 = jvm
                        .invoke_virtual(
                            &choice,
                            "javax/microedition/lcdui/ChoiceGroup",
                            "append",
                            "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
                            (JavaLangString::from_rust_string(&jvm, text).await?, None),
                        )
                        .await?;
                }
                let form = form_with_items(&jvm, vec![JavaValue::from(choice.clone()).into()]).await?;
                let display = show_form(&jvm, &form, 320, 240).await?;
                if choice_type == 4 {
                    send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::FIRE).await?;
                }
                for index in 1..30 {
                    send_key(&jvm, &display, KeyboardEventType::KeyRepeated, MIDPKeyCode::DOWN).await?;
                    assert_eq!(jvm.get_field::<i32>(&choice, "highlightedIndex", "I").await?, index);
                }
                let scroll: i32 = jvm.get_field(&form, "scrollY", "I").await?;
                assert!(scroll > 0, "the final choice must be revealed even without a following Item");
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                assert_eq!(
                    jvm.get_field::<i32>(&form, "scrollY", "I").await?,
                    scroll,
                    "paint must preserve the reveal"
                );
                let mut graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "getScreenGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let image = Graphics::image(&jvm, &mut graphics).await?;
                let pixels = Image::image(&jvm, &image).await?;
                let choice_width: i32 = jvm
                    .invoke_virtual(&choice, "javax/microedition/lcdui/Item", "measureWidth", "(I)I", (320,))
                    .await?;
                assert_eq!(
                    (0..240)
                        .filter(|y| {
                            let pixel = pixels.get_pixel(choice_width - 3, *y);
                            (pixel.r, pixel.g, pixel.b) == (0xdc, 0xec, 0xf7)
                        })
                        .count(),
                    if choice_type == 1 { 13 } else { 14 },
                    "the active element must be fully visible (the inline Item border covers its last pixel)"
                );

                let _: () = jvm
                    .invoke_virtual(
                        &choice,
                        "javax/microedition/lcdui/ChoiceGroup",
                        "set",
                        "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
                        (29, JavaLangString::from_rust_string(&jvm, &"line\n".repeat(30)).await?, None),
                    )
                    .await?;
                let top: i32 = jvm.get_field(&form, "scrollY", "I").await?;
                send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::DOWN).await?;
                assert_eq!(jvm.get_field::<i32>(&choice, "highlightedIndex", "I").await?, 29);
                assert!(
                    jvm.get_field::<i32>(&form, "scrollY", "I").await? > top,
                    "the tall final element's tail must be reachable"
                );
                let tail: i32 = jvm.get_field(&form, "scrollY", "I").await?;
                send_key(&jvm, &display, KeyboardEventType::KeyRepeated, MIDPKeyCode::DOWN).await?;
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                assert!(
                    jvm.get_field::<i32>(&form, "scrollY", "I").await? >= tail,
                    "boundary input and paint must not reset scrolling"
                );
                send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::UP).await?;
                assert_eq!(jvm.get_field::<i32>(&choice, "highlightedIndex", "I").await?, 29);
                for _ in 0..30 {
                    send_key(&jvm, &display, KeyboardEventType::KeyRepeated, MIDPKeyCode::UP).await?;
                }
                assert_eq!(jvm.get_field::<i32>(&choice, "highlightedIndex", "I").await?, 0);
                let _: () = jvm
                    .invoke_virtual(&choice, "javax/microedition/lcdui/ChoiceGroup", "deleteAll", "()V", ())
                    .await?;
                assert_eq!(
                    jvm.get_field::<i32>(&form, "scrollY", "I").await?,
                    0,
                    "content shrink must clamp scrolling"
                );
            }

            Ok(())
        })
    }

    #[test]
    fn form_scrolls_content_without_focusable_items() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let first = recording_item(&jvm, 9, 0x114411, false).await?;
            let tall = recording_item(&jvm, 25, 0x225522, false).await?;
            let last = recording_item(&jvm, 8, 0x336633, false).await?;
            let form = form_with_items(
                &jvm,
                vec![JavaValue::from(first).into(), JavaValue::from(tall).into(), JavaValue::from(last).into()],
            )
            .await?;
            let display = show_form(&jvm, &form, 40, 10).await?;
            assert_eq!(jvm.get_field::<i32>(&form, "focusIndex", "I").await?, -1);

            for _ in 0..8 {
                send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::DOWN).await?;
            }
            assert_eq!(jvm.get_field::<i32>(&form, "scrollY", "I").await?, 32);
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
            assert_eq!(
                jvm.get_field::<i32>(&form, "scrollY", "I").await?,
                32,
                "paint must preserve content scrolling when no Item can take focus"
            );
            for _ in 0..8 {
                send_key(&jvm, &display, KeyboardEventType::KeyRepeated, MIDPKeyCode::UP).await?;
            }
            assert_eq!(jvm.get_field::<i32>(&form, "scrollY", "I").await?, 0);

            Ok(())
        })
    }

    #[test]
    fn focused_item_default_and_menu_commands_dispatch_to_the_item_listener() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let item = recording_item(&jvm, 10, 0x225522, true).await?;
            let item_listener: ClassInstanceRef<RecordingFormItemCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormItemCommandListener", "()V", ())
                .await?
                .into();
            let item_listener_ref: ClassInstanceRef<ItemCommandListener> = JavaValue::from(item_listener.clone()).into();
            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/Item",
                    "setItemCommandListener",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;)V",
                    (item_listener_ref,),
                )
                .await?;
            let menu_command = make_command(&jvm, "Item menu", 8, 0).await?;
            let default_command = make_command(&jvm, "Item default", 8, 1).await?;
            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/Item",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (menu_command.clone(),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/Item",
                    "setDefaultCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (default_command.clone(),),
                )
                .await?;

            let form = form_with_items(&jvm, vec![JavaValue::from(item.clone()).into()]).await?;
            let display_listener: ClassInstanceRef<RecordingFormCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormCommandListener", "()V", ())
                .await?
                .into();
            let display_listener_ref: ClassInstanceRef<CommandListener> = JavaValue::from(display_listener.clone()).into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    (display_listener_ref,),
                )
                .await?;
            let screen_command = make_command(&jvm, "Screen", 1, 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (screen_command.clone(),),
                )
                .await?;

            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&form, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
                    .await?,
                3
            );

            let display = show_form(&jvm, &form, 120, 80).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::FIRE).await?;
            assert_eq!(jvm.get_field::<i32>(&item_listener, "count", "I").await?, 1);
            let dispatched: ClassInstanceRef<Command> = jvm.get_field(&item_listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(dispatched.identity(), default_command.identity());

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::FIRE).await?;
            assert_eq!(jvm.get_field::<i32>(&item_listener, "count", "I").await?, 2);
            let dispatched: ClassInstanceRef<Command> = jvm.get_field(&item_listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(dispatched.identity(), menu_command.identity());
            assert_eq!(jvm.get_field::<i32>(&display_listener, "count", "I").await?, 0);

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::DOWN).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::DOWN).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::FIRE).await?;
            assert_eq!(jvm.get_field::<i32>(&display_listener, "count", "I").await?, 1);
            let dispatched: ClassInstanceRef<Command> = jvm
                .get_field(&display_listener, "lastCommand", "Ljavax/microedition/lcdui/Command;")
                .await?;
            assert_eq!(dispatched.identity(), screen_command.identity());
            assert_eq!(jvm.get_field::<i32>(&item_listener, "count", "I").await?, 2);

            Ok(())
        })
    }

    #[test]
    fn duplicate_effective_command_occurrences_dispatch_to_the_selected_owner() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let item = recording_item(&jvm, 10, 0x225522, true).await?;
            let item_listener: ClassInstanceRef<RecordingFormItemCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormItemCommandListener", "()V", ())
                .await?
                .into();
            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/Item",
                    "setItemCommandListener",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;)V",
                    (ClassInstanceRef::<ItemCommandListener>::from(JavaValue::from(item_listener.clone())),),
                )
                .await?;

            let shared = make_command(&jvm, "Shared", 2, 0).await?;
            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/Item",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (shared.clone(),),
                )
                .await?;
            let form = form_with_items(&jvm, vec![JavaValue::from(item.clone()).into()]).await?;
            let form_listener: ClassInstanceRef<RecordingFormCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormCommandListener", "()V", ())
                .await?
                .into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    (ClassInstanceRef::<CommandListener>::from(JavaValue::from(form_listener.clone())),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (shared.clone(),),
                )
                .await?;

            let display = show_form(&jvm, &form, 120, 80).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::RIGHT_SOFT_KEY).await?;
            assert_eq!(jvm.get_field::<i32>(&item_listener, "count", "I").await?, 1);
            assert_eq!(jvm.get_field::<i32>(&form_listener, "count", "I").await?, 0);

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            assert_eq!(jvm.get_field::<i32>(&item_listener, "count", "I").await?, 1);
            assert_eq!(jvm.get_field::<i32>(&form_listener, "count", "I").await?, 1);
            let item_command: ClassInstanceRef<Command> = jvm.get_field(&item_listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            let form_command: ClassInstanceRef<Command> = jvm.get_field(&form_listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(item_command.identity(), shared.identity());
            assert_eq!(form_command.identity(), shared.identity());

            Ok(())
        })
    }

    #[test]
    fn serial_item_callbacks_run_before_the_next_backend_input_and_command() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let midlet: ClassInstanceRef<RecordingFormMidlet> =
                jvm.new_class("javax/microedition/lcdui/TestRecordingFormMidlet", "()V", ()).await?.into();
            let midlet: ClassInstanceRef<MIDlet> = JavaValue::from(midlet).into();
            let display = MIDlet::display(&jvm, &midlet).await?;

            let first: ClassInstanceRef<Gauge> = jvm
                .new_class("javax/microedition/lcdui/Gauge", "(Ljava/lang/String;ZII)V", (None, true, 1, 0))
                .await?
                .into();
            let second: ClassInstanceRef<Gauge> = jvm
                .new_class("javax/microedition/lcdui/Gauge", "(Ljava/lang/String;ZII)V", (None, true, 1, 0))
                .await?
                .into();
            let form = form_with_items(&jvm, vec![JavaValue::from(first.clone()).into(), JavaValue::from(second.clone()).into()]).await?;
            let state_listener: ClassInstanceRef<RecordingFormItemStateListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormItemStateListener", "()V", ())
                .await?
                .into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Form",
                    "setItemStateListener",
                    "(Ljavax/microedition/lcdui/ItemStateListener;)V",
                    (ClassInstanceRef::<ItemStateListener>::from(JavaValue::from(state_listener.clone())),),
                )
                .await?;
            let command_listener: ClassInstanceRef<RecordingFormCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingFormCommandListener", "()V", ())
                .await?
                .into();
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    (ClassInstanceRef::<CommandListener>::from(JavaValue::from(command_listener.clone())),),
                )
                .await?;
            let command = make_command(&jvm, "Apply", 1, 0).await?;
            let _: () = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command,),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (form.clone(),),
                )
                .await?;

            let _: () = jvm
                .invoke_virtual(&first, "javax/microedition/lcdui/Gauge", "setValue", "(I)V", (0,))
                .await?;
            let event_queue: ClassInstanceRef<EventQueue> = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await?;
            let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
            let _: () = jvm
                .invoke_static("javax/microedition/lcdui/TestBackendEventInjector", "enqueueOrderingInputs", "()V", ())
                .await?;

            for (key, delivered_count) in [
                (MIDPKeyCode::LEFT, 0),
                (MIDPKeyCode::RIGHT, 0),
                (MIDPKeyCode::RIGHT, 1),
                (MIDPKeyCode::DOWN, 1),
                (MIDPKeyCode::RIGHT, 1),
                (MIDPKeyCode::LEFT_SOFT_KEY, 2),
            ] {
                let _: () = jvm
                    .invoke_virtual(&event_queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                    .await?;
                assert_eq!(jvm.load_array::<i32>(&event, 0, 4).await?[2], key as i32);
                assert_eq!(jvm.get_field::<i32>(&state_listener, "count", "I").await?, delivered_count);
                assert_eq!(jvm.get_field::<i32>(&command_listener, "count", "I").await?, 0);
                if delivered_count > 0 {
                    let delivered: ClassInstanceRef<Item> = jvm.get_field(&state_listener, "lastItem", "Ljavax/microedition/lcdui/Item;").await?;
                    assert_eq!(
                        delivered.identity(),
                        if delivered_count == 1 { first.identity() } else { second.identity() }
                    );
                }
                let _: () = jvm
                    .invoke_virtual(&event_queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                    .await?;
                assert_eq!(
                    jvm.get_field::<i32>(&state_listener, "count", "I").await?,
                    delivered_count,
                    "ItemStateListener must not run inside the input handler"
                );
            }
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event,))
                .await?;
            assert_eq!(jvm.get_field::<i32>(&state_listener, "count", "I").await?, 2);
            assert_eq!(jvm.get_field::<i32>(&command_listener, "count", "I").await?, 1);
            Ok(())
        })
    }
}
