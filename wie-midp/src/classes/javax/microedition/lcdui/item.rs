use alloc::string::String as RustString;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_backend::text_layout::{minimum_width, preferred_width, wrap};
use wie_jvm_support::WieJvmContext;

use crate::classes::{
    javax::microedition::lcdui::{Command, Displayable, Font, Graphics, ItemCommandListener},
    net::wie::{CommandEvent, EventQueue},
};

const LABEL_COLOR: i32 = 0x52606d;
const BUTTON_BACKGROUND: i32 = 0xe7edf2;
const BUTTON_BORDER: i32 = 0x65727d;
const FOCUS_BORDER: i32 = 0x2f6f9f;
const LABEL_CONTENT_GAP: i32 = 2;
const BUTTON_INSET: i32 = 3;
const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.Item
#[java_class(name = "javax/microedition/lcdui/Item", public, abstract)]
pub struct Item {
    #[field(public, static, final)]
    LAYOUT_DEFAULT: i32,
    #[field(public, static, final)]
    LAYOUT_LEFT: i32,
    #[field(public, static, final)]
    LAYOUT_RIGHT: i32,
    #[field(public, static, final)]
    LAYOUT_CENTER: i32,
    #[field(public, static, final)]
    LAYOUT_TOP: i32,
    #[field(public, static, final)]
    LAYOUT_BOTTOM: i32,
    #[field(public, static, final)]
    LAYOUT_VCENTER: i32,
    #[field(public, static, final)]
    LAYOUT_NEWLINE_BEFORE: i32,
    #[field(public, static, final)]
    LAYOUT_NEWLINE_AFTER: i32,
    #[field(public, static, final)]
    LAYOUT_SHRINK: i32,
    #[field(public, static, final)]
    LAYOUT_EXPAND: i32,
    #[field(public, static, final)]
    LAYOUT_VSHRINK: i32,
    #[field(public, static, final)]
    LAYOUT_VEXPAND: i32,
    #[field(public, static, final)]
    LAYOUT_2: i32,
    #[field(public, static, final)]
    PLAIN: i32,
    #[field(public, static, final)]
    HYPERLINK: i32,
    #[field(public, static, final)]
    BUTTON: i32,
    #[field(private)]
    owner: ClassInstanceRef<Displayable>,
    #[field(private)]
    label: ClassInstanceRef<String>,
    #[field(private)]
    layout: i32,
    #[field(private)]
    preferredWidth: i32,
    #[field(private)]
    preferredHeight: i32,
    #[field(private)]
    commands: ClassInstanceRef<Vector>,
    #[field(private)]
    defaultCommand: ClassInstanceRef<Command>,
    #[field(private)]
    itemCommandListener: ClassInstanceRef<ItemCommandListener>,
}

#[java_methods]
impl Item {
    pub const INPUT_HANDLED: i32 = 1;
    pub const INPUT_CHANGED: i32 = 2;
    pub const TEXT_COLOR: i32 = 0x17212b;
    pub const LINK_COLOR: i32 = 0x175ca8;

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_DEFAULT", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_LEFT", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_RIGHT", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_CENTER", "I", 3).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_TOP", "I", 0x10).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_BOTTOM", "I", 0x20).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_VCENTER", "I", 0x30).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_NEWLINE_BEFORE", "I", 0x100)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_NEWLINE_AFTER", "I", 0x200)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_SHRINK", "I", 0x400).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_EXPAND", "I", 0x800).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_VSHRINK", "I", 0x1000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_VEXPAND", "I", 0x2000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "LAYOUT_2", "I", 0x4000).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "PLAIN", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "HYPERLINK", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Item", "BUTTON", "I", 2).await?;

        Ok(())
    }

    #[method(constructor)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let commands = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;", commands)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Item", "preferredWidth", "I", -1)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Item", "preferredHeight", "I", -1)
            .await?;

        Ok(())
    }

    #[method]
    async fn get_owner(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Displayable>> {
        tracing::debug!("javax.microedition.lcdui.Item::getOwner({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Item", "owner", "Ljavax/microedition/lcdui/Displayable;")
            .await
    }

    #[method]
    async fn set_owner(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        owner: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setOwner({this:?}, {owner:?})");

        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Item",
            "owner",
            "Ljavax/microedition/lcdui/Displayable;",
            owner,
        )
        .await
    }

    #[method]
    async fn get_command_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getCommandCount({this:?})");

        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
        jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await
    }

    #[method]
    async fn get_command_at(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
    ) -> JvmResult<ClassInstanceRef<Command>> {
        tracing::debug!("javax.microedition.lcdui.Item::getCommandAt({this:?}, {index})");

        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
        jvm.invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    #[method]
    async fn dispatch_default_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::dispatchDefaultCommand({this:?})");

        let command: ClassInstanceRef<Command> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/Item",
                "defaultCommand",
                "Ljavax/microedition/lcdui/Command;",
            )
            .await?;
        if !command.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &this,
                    "javax/microedition/lcdui/Item",
                    "dispatchCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command,),
                )
                .await?;
        }
        Ok(())
    }

    #[method]
    async fn has_preferred_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Item::hasPreferredSize({this:?})");

        let width: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "preferredWidth", "I").await?;
        let height: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "preferredHeight", "I").await?;
        Ok(width != -1 || height != -1)
    }

    #[method]
    async fn can_be_alert_indicator(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Item::canBeAlertIndicator({this:?})");

        let owner: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "owner", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
        let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
        let listener: ClassInstanceRef<ItemCommandListener> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/Item",
                "itemCommandListener",
                "Ljavax/microedition/lcdui/ItemCommandListener;",
            )
            .await?;
        let label: ClassInstanceRef<String> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "label", "Ljava/lang/String;")
            .await?;
        let preferred: bool = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "hasPreferredSize", "()Z", ())
            .await?;
        let layout: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "layout", "I").await?;
        Ok(owner.is_null() && command_count == 0 && listener.is_null() && label.is_null() && !preferred && layout == 0)
    }

    async fn check_owner_mutation(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<()> {
        let owner: ClassInstanceRef<Displayable> = jvm
            .get_field(this, "javax/microedition/lcdui/Item", "owner", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
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

    #[method]
    async fn invalidate(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, layout_changed: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::invalidate({this:?}, {layout_changed})");

        let owner: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "owner", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
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

    #[method(public)]
    async fn add_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::addCommand({this:?}, {command:?})");

        Self::check_owner_mutation(jvm, &this).await?;
        if command.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Command is null").await);
        }

        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
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
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
                .await?;
        }

        Ok(())
    }

    #[method(public)]
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

        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
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
            let default_command: ClassInstanceRef<Command> = jvm
                .get_field(
                    &this,
                    "javax/microedition/lcdui/Item",
                    "defaultCommand",
                    "Ljavax/microedition/lcdui/Command;",
                )
                .await?;
            if !default_command.is_null() && default_command.identity() == command.identity() {
                jvm.put_field(
                    &mut this,
                    "javax/microedition/lcdui/Item",
                    "defaultCommand",
                    "Ljavax/microedition/lcdui/Command;",
                    None,
                )
                .await?;
            }
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
                .await?;
            break;
        }

        Ok(())
    }

    #[method(public)]
    async fn get_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Item::getLabel({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Item", "label", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn set_label(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setLabel({this:?}, {label:?})");

        Self::check_owner_mutation(jvm, &this).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Item", "label", "Ljava/lang/String;", label)
            .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    #[method(public)]
    async fn get_layout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getLayout({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Item", "layout", "I").await
    }

    #[method(public)]
    async fn set_layout(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, layout: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setLayout({this:?}, {layout})");

        Self::check_owner_mutation(jvm, &this).await?;
        if layout & !0x7f33 != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Item layout").await);
        }

        jvm.put_field(&mut this, "javax/microedition/lcdui/Item", "layout", "I", layout).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    #[method(public)]
    async fn get_minimum_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getMinimumWidth({this:?})");

        let content_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "minimumContentWidth", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_width = label
            .as_deref()
            .map(|text| minimum_width(context.system().platform().font(), text, 10.0))
            .unwrap_or(0);
        Ok(content_width.max(label_width))
    }

    #[method(public)]
    async fn get_minimum_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getMinimumHeight({this:?})");

        let content_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "minimumContentHeight", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_height = label
            .as_deref()
            .map(|label| wrap(context.system().platform().font(), label, 10.0, None).len() as i32 * Font::HEIGHT)
            .unwrap_or(0);
        Ok(label_height + content_height + if label_height > 0 && content_height > 0 { LABEL_CONTENT_GAP } else { 0 })
    }

    #[method(public)]
    async fn get_preferred_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getPreferredWidth({this:?})");

        let locked_width: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "preferredWidth", "I").await?;
        let minimum_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumWidth", "()I", ())
            .await?;
        if locked_width >= 0 {
            return Ok(locked_width.max(minimum_width));
        }

        let content_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "preferredContentWidth", "()I", ())
            .await?;
        let label = Self::label_text(jvm, &this).await?;
        let label_width = label
            .as_deref()
            .map(|text| preferred_width(context.system().platform().font(), text, 10.0))
            .unwrap_or(0);
        Ok(content_width.max(label_width).max(minimum_width))
    }

    #[method(public)]
    async fn get_preferred_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::getPreferredHeight({this:?})");

        let preferred_height: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "preferredHeight", "I").await?;
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

    #[method(public)]
    async fn set_preferred_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setPreferredSize({this:?}, {width}, {height})");

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
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Item",
            "preferredWidth",
            "I",
            if width < 0 { -1 } else { width.max(minimum_width) },
        )
        .await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Item",
            "preferredHeight",
            "I",
            if height < 0 { -1 } else { height.max(minimum_height) },
        )
        .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    #[method(public)]
    async fn set_default_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setDefaultCommand({this:?}, {command:?})");

        Self::check_owner_mutation(jvm, &this).await?;

        if !command.is_null() {
            let commands: ClassInstanceRef<Vector> = jvm
                .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
                .await?;
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

        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Item",
            "defaultCommand",
            "Ljavax/microedition/lcdui/Command;",
            command,
        )
        .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await
    }

    #[method(public)]
    async fn set_item_command_listener(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemCommandListener>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::setItemCommandListener({this:?}, {listener:?})");

        Self::check_owner_mutation(jvm, &this).await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Item",
            "itemCommandListener",
            "Ljavax/microedition/lcdui/ItemCommandListener;",
            listener,
        )
        .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await
    }

    #[method(public)]
    async fn notify_state_changed(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::notifyStateChanged({this:?})");

        let owner: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "owner", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
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

    #[method]
    async fn measure_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, available_width: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::measureWidth({this:?}, {available_width})");

        let layout: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "layout", "I").await?;
        if layout & 0x800 != 0 {
            return Ok(available_width.max(0));
        }
        let preferred_width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
            .await?;
        Ok(preferred_width.min(available_width.max(0)))
    }

    #[method]
    async fn measure_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::measureHeight({this:?}, {width})");

        let minimum_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getMinimumHeight", "()I", ())
            .await?;
        let preferred_height: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Item", "preferredHeight", "I").await?;
        if preferred_height >= 0 {
            return Ok(preferred_height.max(minimum_height));
        }

        let available_width = if width < 0 {
            jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "getPreferredWidth", "()I", ())
                .await?
        } else {
            width
        };
        Ok(Self::natural_height(jvm, context, &this, available_width).await?.max(minimum_height))
    }

    #[allow(clippy::too_many_arguments)]
    #[method]
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
        tracing::debug!("javax.microedition.lcdui.Item::paintItem({this:?}, {graphics:?}, {x}, {y}, {width}, {height}, {focused})");

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
            .map(|label| wrap(context.system().platform().font(), label, 10.0, Some(width)))
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

    #[method]
    async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::minimumContentWidth({_this:?})");

        Ok(0)
    }

    #[method]
    async fn minimum_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::minimumContentHeight({_this:?})");

        Ok(0)
    }

    #[method]
    async fn preferred_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::preferredContentWidth({_this:?})");

        Ok(0)
    }

    #[method]
    async fn preferred_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _width: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::preferredContentHeight({_this:?}, {_width})");

        Ok(0)
    }

    #[allow(clippy::too_many_arguments)]
    #[method]
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
        tracing::debug!("javax.microedition.lcdui.Item::paintContent({_this:?}, {_graphics:?}, {_x}, {_y}, {_width}, {_height}, {_focused})");

        Ok(())
    }

    #[method]
    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Item::isFocusable({this:?})");

        let commands: ClassInstanceRef<Vector> = jvm
            .get_field(&this, "javax/microedition/lcdui/Item", "commands", "Ljava/util/Vector;")
            .await?;
        Ok(jvm.invoke_virtual::<_, i32>(&commands, "java/util/Vector", "size", "()I", ()).await? > 0)
    }

    #[method]
    async fn handle_item_key(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Item::handleItemKey({_this:?}, {_key})");

        Ok(0)
    }

    // Null selects the whole Item; otherwise the pair is content-local [top, bottom).
    #[method]
    async fn get_focus_content_bounds(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _width: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::debug!("javax.microedition.lcdui.Item::getFocusContentBounds({_this:?}, {_width})");

        Ok(None.into())
    }

    #[method(name = "getFocusBounds")]
    async fn focus_bounds(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        width: i32,
        height: i32,
    ) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::debug!("javax.microedition.lcdui.Item::getFocusBounds({this:?}, {width}, {height})");

        let bounds: ClassInstanceRef<Array<i32>> = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "getFocusContentBounds", "(I)[I", (width,))
            .await?;
        let bounds = if bounds.is_null() {
            [0, height]
        } else {
            let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
            let label = Self::label_text(jvm, &this).await?;
            let label_height = label
                .as_deref()
                .map(|label| wrap(context.system().platform().font(), label, 10.0, Some(width)).len() as i32 * Font::HEIGHT)
                .unwrap_or(0)
                .min(height);
            let content_y = label_height
                + if label_height > 0 {
                    LABEL_CONTENT_GAP.min(height - label_height)
                } else {
                    0
                };
            [(content_y + bounds[0]).min(height), (content_y + bounds[1]).min(height)]
        };
        let mut result = jvm.instantiate_array("I", 2).await?;
        jvm.store_array(&mut result, 0, bounds).await?;
        Ok(result.into())
    }

    #[method]
    async fn dispatch_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::dispatchCommand({this:?}, {command:?})");

        let listener: ClassInstanceRef<ItemCommandListener> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/Item",
                "itemCommandListener",
                "Ljavax/microedition/lcdui/ItemCommandListener;",
            )
            .await?;
        if !listener.is_null() {
            let event: ClassInstanceRef<CommandEvent> = jvm
                .new_class(
                    "net/wie/CommandEvent",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Item;)V",
                    (listener, command, this),
                )
                .await?
                .into();
            let queue: ClassInstanceRef<EventQueue> = jvm
                .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(&queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (event,))
                .await?;
        }

        Ok(())
    }

    async fn label_text<T: Sync>(jvm: &Jvm, this: &ClassInstanceRef<T>) -> JvmResult<Option<RustString>> {
        let label: ClassInstanceRef<String> = jvm
            .get_field(this, "javax/microedition/lcdui/Item", "label", "Ljava/lang/String;")
            .await?;
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
            .map(|label| wrap(context.system().platform().font(), label, 10.0, Some(width.max(1))).len() as i32 * Font::HEIGHT)
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
    #[method(static)]
    async fn paint_appearance(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        appearance_mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Item::paintAppearance({graphics:?}, {x}, {y}, {width}, {height}, {appearance_mode})");

        if width <= 0 || height <= 0 {
            return Ok(());
        }

        match appearance_mode {
            1 => {
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (Self::LINK_COLOR,))
                    .await?;
                jvm.invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawLine",
                    "(IIII)V",
                    (x, y + height - 1, x + width - 1, y + height - 1),
                )
                .await
            }
            2 => {
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BUTTON_BACKGROUND,))
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
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BUTTON_BORDER,))
                    .await?;
                jvm.invoke_virtual(
                    &graphics,
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
                    (None, text.clone()),
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
