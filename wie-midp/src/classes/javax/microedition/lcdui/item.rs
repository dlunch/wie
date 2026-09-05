use alloc::{string::String as RustString, vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Command, Displayable, Font, Graphics, ItemCommandListener};

const LABEL_COLOR: i32 = 0x52606d;
const BUTTON_BACKGROUND: i32 = 0xe7edf2;
const BUTTON_BORDER: i32 = 0x65727d;
const FOCUS_BORDER: i32 = 0x2f6f9f;
const LABEL_CONTENT_GAP: i32 = 2;
const BUTTON_INSET: i32 = 3;
const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.Item
pub struct Item;

impl Item {
    pub const INPUT_HANDLED: i32 = 1;
    pub const INPUT_CHANGED: i32 = 2;
    pub const TEXT_COLOR: i32 = 0x17212b;
    pub const LINK_COLOR: i32 = 0x175ca8;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Item",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::add_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "removeCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::remove_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getLabel", "()Ljava/lang/String;", Self::get_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLabel", "(Ljava/lang/String;)V", Self::set_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getLayout", "()I", Self::get_layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLayout", "(I)V", Self::set_layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMinimumWidth", "()I", Self::get_minimum_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMinimumHeight", "()I", Self::get_minimum_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPreferredWidth", "()I", Self::get_preferred_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPreferredHeight", "()I", Self::get_preferred_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPreferredSize", "(II)V", Self::set_preferred_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setDefaultCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::set_default_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setItemCommandListener",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;)V",
                    Self::set_item_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("notifyStateChanged", "()V", Self::notify_state_changed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("measureHeight", "(I)I", Self::measure_height, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "paintItem",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    Self::paint_item,
                    MethodAccessFlags::empty(),
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
                JavaMethodProto::new(
                    "getFocusContentBounds",
                    "(I)[I",
                    Self::get_focus_content_bounds,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "dispatchCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::dispatch_command,
                    MethodAccessFlags::empty(),
                ),
            ],
            fields: [
                "LAYOUT_DEFAULT",
                "LAYOUT_LEFT",
                "LAYOUT_RIGHT",
                "LAYOUT_CENTER",
                "LAYOUT_TOP",
                "LAYOUT_BOTTOM",
                "LAYOUT_VCENTER",
                "LAYOUT_NEWLINE_BEFORE",
                "LAYOUT_NEWLINE_AFTER",
                "LAYOUT_SHRINK",
                "LAYOUT_EXPAND",
                "LAYOUT_VSHRINK",
                "LAYOUT_VEXPAND",
                "LAYOUT_2",
                "PLAIN",
                "HYPERLINK",
                "BUTTON",
            ]
            .into_iter()
            .map(|name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
            .chain([
                JavaFieldProto::new("owner", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("label", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("layout", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("preferredWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("preferredHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("commands", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("defaultCommand", "Ljavax/microedition/lcdui/Command;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "itemCommandListener",
                    "Ljavax/microedition/lcdui/ItemCommandListener;",
                    FieldAccessFlags::PRIVATE,
                ),
            ])
            .collect(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::<clinit>");

        for (name, value) in [
            ("LAYOUT_DEFAULT", 0),
            ("LAYOUT_LEFT", 1),
            ("LAYOUT_RIGHT", 2),
            ("LAYOUT_CENTER", 3),
            ("LAYOUT_TOP", 0x10),
            ("LAYOUT_BOTTOM", 0x20),
            ("LAYOUT_VCENTER", 0x30),
            ("LAYOUT_NEWLINE_BEFORE", 0x100),
            ("LAYOUT_NEWLINE_AFTER", 0x200),
            ("LAYOUT_SHRINK", 0x400),
            ("LAYOUT_EXPAND", 0x800),
            ("LAYOUT_VSHRINK", 0x1000),
            ("LAYOUT_VEXPAND", 0x2000),
            ("LAYOUT_2", 0x4000),
            ("PLAIN", 0),
            ("HYPERLINK", 1),
            ("BUTTON", 2),
        ] {
            jvm.put_static_field("javax/microedition/lcdui/Item", name, "I", value).await?;
        }

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let commands = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "commands", "Ljava/util/Vector;", commands).await?;
        jvm.put_field(&mut this, "preferredWidth", "I", -1).await?;
        jvm.put_field(&mut this, "preferredHeight", "I", -1).await?;

        Ok(())
    }

    async fn check_owner_mutation(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<()> {
        let owner: ClassInstanceRef<Displayable> = jvm.get_field(this, "owner", "Ljavax/microedition/lcdui/Displayable;").await?;
        if !owner.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &owner,
                    "javax/microedition/lcdui/Displayable",
                    "checkItemMutation",
                    "(Ljavax/microedition/lcdui/Item;)V",
                    (this.clone(),),
                )
                .await?;
        }

        Ok(())
    }

    pub async fn invalidate<T: Send + Sync + 'static>(jvm: &Jvm, this: &ClassInstanceRef<T>, layout_changed: bool) -> JvmResult<()> {
        let owner: ClassInstanceRef<Displayable> = jvm.get_field(this, "owner", "Ljavax/microedition/lcdui/Displayable;").await?;
        if !owner.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &owner,
                    "javax/microedition/lcdui/Displayable",
                    "itemInvalidated",
                    "(Ljavax/microedition/lcdui/Item;Z)V",
                    (this.clone(), layout_changed),
                )
                .await?;
        }

        Ok(())
    }

    async fn add_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::addCommand({this:?}, {command:?})");

        Self::check_owner_mutation(jvm, &this).await?;
        if command.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Command is null").await);
        }

        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
        let mut registered = false;
        for index in 0..command_count {
            let existing: ClassInstanceRef<Command> = jvm
                .invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                .await?;
            if existing.identity() == command.identity() {
                registered = true;
                break;
            }
        }
        if !registered {
            let _: () = jvm
                .invoke_virtual(&commands, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (command,))
                .await?;
            Self::invalidate(jvm, &this, false).await?;
        }

        Ok(())
    }

    async fn remove_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::removeCommand({this:?}, {command:?})");

        Self::check_owner_mutation(jvm, &this).await?;
        if command.is_null() {
            return Ok(());
        }

        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
        for index in 0..command_count {
            let existing: ClassInstanceRef<Command> = jvm
                .invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                .await?;
            if existing.identity() != command.identity() {
                continue;
            }

            let _: () = jvm
                .invoke_virtual(&commands, "java/util/Vector", "removeElementAt", "(I)V", (index,))
                .await?;
            let default_command: ClassInstanceRef<Command> = jvm.get_field(&this, "defaultCommand", "Ljavax/microedition/lcdui/Command;").await?;
            if !default_command.is_null() && default_command.identity() == command.identity() {
                jvm.put_field(
                    &mut this,
                    "defaultCommand",
                    "Ljavax/microedition/lcdui/Command;",
                    ClassInstanceRef::<Command>::new(None),
                )
                .await?;
            }
            Self::invalidate(jvm, &this, false).await?;
            break;
        }

        Ok(())
    }

    async fn get_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "label", "Ljava/lang/String;").await
    }

    async fn set_label(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        Self::check_owner_mutation(jvm, &this).await?;
        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        Self::invalidate(jvm, &this, true).await
    }

    async fn get_layout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "layout", "I").await
    }

    async fn set_layout(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, layout: i32) -> JvmResult<()> {
        Self::check_owner_mutation(jvm, &this).await?;
        if layout & !0x7f33 != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Item layout").await);
        }

        jvm.put_field(&mut this, "layout", "I", layout).await?;
        Self::invalidate(jvm, &this, true).await
    }

    async fn get_minimum_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let content_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "minimumContentWidth", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_width = label
            .as_deref()
            .map(|text| Font::minimum_width(context.system().platform().font(), text))
            .unwrap_or(0);
        Ok(content_width.max(label_width))
    }

    async fn get_minimum_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let content_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "minimumContentHeight", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_height = label
            .as_deref()
            .map(|label| Font::wrap(context.system().platform().font(), label, None).len() as i32 * Font::HEIGHT)
            .unwrap_or(0);
        Ok(label_height + content_height + if label_height > 0 && content_height > 0 { LABEL_CONTENT_GAP } else { 0 })
    }

    async fn get_preferred_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let preferred_width: i32 = jvm.get_field(&this, "preferredWidth", "I").await?;
        let minimum_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumWidth", "()I", ())
            .await?;
        if preferred_width >= 0 {
            return Ok(preferred_width.max(minimum_width));
        }

        let content_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "preferredContentWidth", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_width = label
            .as_deref()
            .map(|text| Font::preferred_width(context.system().platform().font(), text))
            .unwrap_or(0);
        Ok(content_width.max(label_width).max(minimum_width))
    }

    async fn get_preferred_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let preferred_height: i32 = jvm.get_field(&this, "preferredHeight", "I").await?;
        let minimum_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumHeight", "()I", ())
            .await?;
        if preferred_height >= 0 {
            return Ok(preferred_height.max(minimum_height));
        }

        let preferred_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
            .await?;
        Ok(Self::natural_height(jvm, context, &this, preferred_width).await?.max(minimum_height))
    }

    async fn set_preferred_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        Self::check_owner_mutation(jvm, &this).await?;
        if width < -1 || height < -1 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Preferred size is less than -1")
                .await);
        }

        let minimum_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumWidth", "()I", ())
            .await?;
        let minimum_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumHeight", "()I", ())
            .await?;
        jvm.put_field(&mut this, "preferredWidth", "I", if width < 0 { -1 } else { width.max(minimum_width) })
            .await?;
        jvm.put_field(
            &mut this,
            "preferredHeight",
            "I",
            if height < 0 { -1 } else { height.max(minimum_height) },
        )
        .await?;
        Self::invalidate(jvm, &this, true).await
    }

    async fn set_default_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        Self::check_owner_mutation(jvm, &this).await?;

        if !command.is_null() {
            let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
            let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
            let mut registered = false;
            for index in 0..command_count {
                let existing: ClassInstanceRef<Command> = jvm
                    .invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                    .await?;
                if existing.identity() == command.identity() {
                    registered = true;
                    break;
                }
            }
            if !registered {
                let _: () = jvm
                    .invoke_virtual(&commands, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (command.clone(),))
                    .await?;
            }
        }

        jvm.put_field(&mut this, "defaultCommand", "Ljavax/microedition/lcdui/Command;", command)
            .await?;
        Self::invalidate(jvm, &this, false).await
    }

    async fn set_item_command_listener(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemCommandListener>,
    ) -> JvmResult<()> {
        Self::check_owner_mutation(jvm, &this).await?;
        jvm.put_field(
            &mut this,
            "itemCommandListener",
            "Ljavax/microedition/lcdui/ItemCommandListener;",
            listener,
        )
        .await?;
        Self::invalidate(jvm, &this, false).await
    }

    async fn notify_state_changed(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let owner: ClassInstanceRef<Displayable> = jvm.get_field(&this, "owner", "Ljavax/microedition/lcdui/Displayable;").await?;
        if owner.is_null() {
            return Err(jvm.exception("java/lang/IllegalStateException", "Item is not owned by a Form").await);
        }

        jvm.invoke_virtual(
            &owner,
            "javax/microedition/lcdui/Displayable",
            "itemStateChanged",
            "(Ljavax/microedition/lcdui/Item;)V",
            (this,),
        )
        .await
    }

    async fn measure_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        let minimum_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumHeight", "()I", ())
            .await?;
        let preferred_height: i32 = jvm.get_field(&this, "preferredHeight", "I").await?;
        if preferred_height >= 0 {
            return Ok(preferred_height.max(minimum_height));
        }

        let available_width = if width < 0 {
            jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                .await?
        } else {
            width
        };
        let preferred_width: i32 = jvm.get_field(&this, "preferredWidth", "I").await?;
        let measured_width = if preferred_width >= 0 {
            let minimum_width: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumWidth", "()I", ())
                .await?;
            available_width.min(preferred_width.max(minimum_width))
        } else {
            available_width
        };
        Ok(Self::natural_height(jvm, context, &this, measured_width).await?.max(minimum_height))
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_item(
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
        let width = width.max(0);
        let height = height.max(0);
        if width == 0 || height == 0 {
            return Ok(());
        }

        let clip_x: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
            .await?;
        let clip_y: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
            .await?;
        let clip_width: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
            .await?;
        let clip_height: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "clipRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await?;

        let label = Self::label_text(jvm, &this).await?;
        let label_lines = label
            .as_deref()
            .map(|label| Font::wrap(context.system().platform().font(), label, Some(width)))
            .unwrap_or_default();
        let label_height = (label_lines.len() as i32 * Font::HEIGHT).min(height);
        if !label_lines.is_empty() {
            let default_font: ClassInstanceRef<Font> = jvm
                .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "setFont",
                    "(Ljavax/microedition/lcdui/Font;)V",
                    (default_font,),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (LABEL_COLOR,))
                .await?;
            for (index, line) in label_lines.iter().enumerate() {
                let line_y = y + index as i32 * Font::HEIGHT;
                if line_y >= y + label_height {
                    break;
                }
                let line = JavaLangString::from_rust_string(jvm, line).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawString",
                        "(Ljava/lang/String;III)V",
                        (line, x, line_y, LEFT_TOP),
                    )
                    .await?;
            }
        }

        let content_preferred_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "preferredContentHeight", "(I)I", (width,))
            .await?;
        let gap = if label_height > 0 && content_preferred_height > 0 {
            LABEL_CONTENT_GAP.min((height - label_height).max(0))
        } else {
            0
        };
        let content_y = y + label_height + gap;
        let content_height = (height - label_height - gap).max(0);
        let paint_result: JvmResult<()> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Item",
                "paintContent",
                "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                (graphics.clone(), x, content_y, width, content_height, focused),
            )
            .await;

        if paint_result.is_ok() && focused {
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (FOCUS_BORDER,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawRect",
                    "(IIII)V",
                    (x, y, (width - 1).max(0), (height - 1).max(0)),
                )
                .await?;
        }

        let restore_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setClip",
                "(IIII)V",
                (clip_x, clip_y, clip_width, clip_height),
            )
            .await;
        paint_result?;
        restore_result
    }

    async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(0)
    }

    async fn minimum_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(0)
    }

    async fn preferred_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(0)
    }

    async fn preferred_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _width: i32) -> JvmResult<i32> {
        Ok(0)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _graphics: ClassInstanceRef<Graphics>,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _focused: bool,
    ) -> JvmResult<()> {
        Ok(())
    }

    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        Ok(jvm.invoke_virtual::<_, i32>(&commands, "java/util/Vector", "size", "()I", ()).await? > 0)
    }

    async fn handle_item_key(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _key: i32) -> JvmResult<i32> {
        Ok(0)
    }

    // Null selects the whole Item; otherwise the pair is content-local [top, bottom).
    async fn get_focus_content_bounds(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _width: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        Ok(ClassInstanceRef::new(None))
    }

    pub async fn focus_bounds(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: &ClassInstanceRef<Self>,
        width: i32,
        height: i32,
    ) -> JvmResult<(i32, i32)> {
        let bounds: ClassInstanceRef<Array<i32>> = jvm
            .invoke_virtual(this, "javax/microedition/lcdui/Item", "getFocusContentBounds", "(I)[I", (width,))
            .await?;
        if bounds.is_null() {
            return Ok((0, height));
        }

        let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
        let label = Self::label_text(jvm, this).await?;
        let label_height = label
            .as_deref()
            .map(|label| Font::wrap(context.system().platform().font(), label, Some(width)).len() as i32 * Font::HEIGHT)
            .unwrap_or(0)
            .min(height);
        let content_y = label_height
            + if label_height > 0 {
                LABEL_CONTENT_GAP.min(height - label_height)
            } else {
                0
            };
        Ok(((content_y + bounds[0]).min(height), (content_y + bounds[1]).min(height)))
    }

    async fn dispatch_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        let listener: ClassInstanceRef<ItemCommandListener> = jvm
            .get_field(&this, "itemCommandListener", "Ljavax/microedition/lcdui/ItemCommandListener;")
            .await?;
        if !listener.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &listener,
                    "javax/microedition/lcdui/ItemCommandListener",
                    "commandAction",
                    "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Item;)V",
                    (command, this),
                )
                .await?;
        }

        Ok(())
    }

    async fn label_text<T>(jvm: &Jvm, this: &ClassInstanceRef<T>) -> JvmResult<Option<RustString>> {
        let label: ClassInstanceRef<String> = jvm.get_field(this, "label", "Ljava/lang/String;").await?;
        if label.is_null() {
            return Ok(None);
        }

        let label = JavaLangString::to_rust_string(jvm, &label).await?;
        Ok((!label.is_empty()).then_some(label))
    }

    async fn natural_height(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        let label = Self::label_text(jvm, this).await?;
        let label_height = label
            .as_deref()
            .map(|label| Font::wrap(context.system().platform().font(), label, Some(width.max(1))).len() as i32 * Font::HEIGHT)
            .unwrap_or(0);
        let content_height: i32 = jvm
            .invoke_virtual(this, "javax/microedition/lcdui/Item", "preferredContentHeight", "(I)I", (width,))
            .await?;
        Ok(label_height + content_height + if label_height > 0 && content_height > 0 { LABEL_CONTENT_GAP } else { 0 })
    }

    pub fn appearance_inset(appearance_mode: i32) -> i32 {
        if appearance_mode == 2 { BUTTON_INSET } else { 0 }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn paint_appearance(
        jvm: &Jvm,
        graphics: &ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        appearance_mode: i32,
    ) -> JvmResult<()> {
        if width <= 0 || height <= 0 {
            return Ok(());
        }

        match appearance_mode {
            1 => {
                let _: () = jvm
                    .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (Self::LINK_COLOR,))
                    .await?;
                jvm.invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawLine",
                    "(IIII)V",
                    (x, y + height - 1, x + width - 1, y + height - 1),
                )
                .await
            }
            2 => {
                let _: () = jvm
                    .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BUTTON_BACKGROUND,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        graphics,
                        "javax/microedition/lcdui/Graphics",
                        "fillRect",
                        "(IIII)V",
                        (x, y, width, height),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BUTTON_BORDER,))
                    .await?;
                jvm.invoke_virtual(
                    graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawRect",
                    "(IIII)V",
                    (x, y, width - 1, height - 1),
                )
                .await
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::lcdui::StringItem, get_protos};

    #[test]
    fn item_sizes_include_labels_wrap_text_and_honor_dimension_locks() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let text = ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "alpha beta gamma delta epsilon").await?);
            let unlabeled: ClassInstanceRef<StringItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/StringItem",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    (ClassInstanceRef::<String>::new(None), text.clone()),
                )
                .await?
                .into();
            let label = ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "First label line\nSecond label line").await?);
            let labeled: ClassInstanceRef<StringItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/StringItem",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    (label, text),
                )
                .await?
                .into();

            let minimum_width: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "getMinimumWidth", "()I", ())
                .await?;
            let minimum_height: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "getMinimumHeight", "()I", ())
                .await?;
            let natural_width: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                .await?;
            let natural_height: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredHeight", "()I", ())
                .await?;
            assert!(minimum_width > 0);
            assert!(minimum_height > 0);
            assert!(natural_width > minimum_width);
            assert!(natural_height >= minimum_height);

            let wide_height: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (natural_width,))
                .await?;
            let narrow_height: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (minimum_width + 8,))
                .await?;
            assert!(narrow_height > wide_height, "word and character wrapping must increase row height");

            let labeled_height: i32 = jvm
                .invoke_virtual(&labeled, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (natural_width,))
                .await?;
            assert!(
                labeled_height >= wide_height + 24,
                "two explicit label lines must contribute to row height"
            );
            assert!(
                jvm.invoke_virtual::<_, i32>(&labeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                    .await?
                    >= natural_width
            );

            let _: () = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/StringItem", "setPreferredSize", "(II)V", (0, 0))
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                    .await?,
                minimum_width
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredHeight", "()I", ())
                    .await?,
                minimum_height
            );

            let locked_width = minimum_width + 8;
            let _: () = jvm
                .invoke_virtual(
                    &unlabeled,
                    "javax/microedition/lcdui/StringItem",
                    "setPreferredSize",
                    "(II)V",
                    (locked_width, -1),
                )
                .await?;
            let locked_height: i32 = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredHeight", "()I", ())
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                    .await?,
                locked_width
            );
            assert!(locked_height > natural_height);

            let longer_text =
                ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "alpha beta gamma delta epsilon zeta eta theta iota").await?);
            let _: () = jvm
                .invoke_virtual(
                    &unlabeled,
                    "javax/microedition/lcdui/StringItem",
                    "setText",
                    "(Ljava/lang/String;)V",
                    (longer_text,),
                )
                .await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                    .await?,
                locked_width
            );
            assert!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredHeight", "()I", ())
                    .await?
                    > locked_height
            );

            let _: () = jvm
                .invoke_virtual(&unlabeled, "javax/microedition/lcdui/StringItem", "setPreferredSize", "(II)V", (-1, -1))
                .await?;
            assert!(
                jvm.invoke_virtual::<_, i32>(&unlabeled, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                    .await?
                    > locked_width
            );

            Ok(())
        })
    }
}
