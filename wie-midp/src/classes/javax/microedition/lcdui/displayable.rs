use alloc::{format, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Command, CommandListener, Display, Graphics, Item, Ticker};

// class javax.microedition.lcdui.Displayable
pub struct Displayable;

impl Displayable {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Displayable",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("getTitle", "()Ljava/lang/String;", Self::get_title, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTitle", "(Ljava/lang/String;)V", Self::set_title, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getTicker",
                    "()Ljavax/microedition/lcdui/Ticker;",
                    Self::get_ticker,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setTicker",
                    "(Ljavax/microedition/lcdui/Ticker;)V",
                    Self::set_ticker,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
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
                JavaMethodProto::new(
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    Self::set_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("sizeChanged", "(II)V", Self::size_changed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "setDisplay",
                    "(Ljavax/microedition/lcdui/Display;)V",
                    Self::set_display,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("requestRepaint", "()V", Self::request_repaint, MethodAccessFlags::empty()),
                JavaMethodProto::new("decorationChanged", "()V", Self::decoration_changed, MethodAccessFlags::empty()),
                JavaMethodProto::new("notifySizeChanged", "()V", Self::notify_size_changed, MethodAccessFlags::empty()),
                JavaMethodProto::new("getCommandCount", "()I", Self::get_command_count, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "getCommandAt",
                    "(I)Ljavax/microedition/lcdui/Command;",
                    Self::get_command_at,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("dispatchCommandAt", "(I)V", Self::dispatch_command_at, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "checkItemMutation",
                    "(Ljavax/microedition/lcdui/Item;)V",
                    Self::check_item_mutation,
                    MethodAccessFlags::empty(),
                ),
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
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::handle_paint_event,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("handleNotifyEvent", "(III)V", Self::handle_notify_event, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("title", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("ticker", "Ljavax/microedition/lcdui/Ticker;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("commands", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("commandListener", "Ljavax/microedition/lcdui/CommandListener;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("currentDisplay", "Ljavax/microedition/lcdui/Display;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("notifiedWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("notifiedHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("sizeKnown", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("sizeDirty", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("sizeNotifying", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("commandMenuOpen", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("commandMenuIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("isInFullScreenMode", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let commands = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "commands", "Ljava/util/Vector;", commands).await?;
        jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
        jvm.put_field(&mut this, "commandMenuIndex", "I", -1).await?;

        Ok(())
    }

    async fn get_title(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "title", "Ljava/lang/String;").await
    }

    async fn set_title(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.put_field(&mut this, "title", "Ljava/lang/String;", title).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn get_ticker(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Ticker>> {
        jvm.get_field(&this, "ticker", "Ljavax/microedition/lcdui/Ticker;").await
    }

    async fn set_ticker(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, ticker: ClassInstanceRef<Ticker>) -> JvmResult<()> {
        let previous: ClassInstanceRef<Ticker> = jvm.get_field(&this, "ticker", "Ljavax/microedition/lcdui/Ticker;").await?;
        if (previous.is_null() && ticker.is_null()) || (!previous.is_null() && !ticker.is_null() && previous.identity() == ticker.identity()) {
            return Ok(());
        }
        jvm.put_field(&mut this, "ticker", "Ljavax/microedition/lcdui/Ticker;", ticker).await?;
        if Self::is_shown(jvm, context, this.clone()).await? {
            let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "tickerChanged", "()V", ())
                .await?;
        }
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await
    }

    async fn is_shown(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if display.is_null() {
            return Ok(false);
        }

        let current: ClassInstanceRef<Displayable> = jvm
            .invoke_virtual(
                &display,
                "javax/microedition/lcdui/Display",
                "getCurrent",
                "()Ljavax/microedition/lcdui/Displayable;",
                (),
            )
            .await?;
        Ok(!current.is_null() && current.identity() == this.identity())
    }

    async fn add_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::addCommand({this:?}, {command:?})");

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
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
                .await?;
        }

        Ok(())
    }

    async fn remove_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::removeCommand({this:?}, {command:?})");

        if command.is_null() {
            return Ok(());
        }

        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
        for index in 0..command_count {
            let existing: ClassInstanceRef<Command> = jvm
                .invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                .await?;
            if existing.identity() == command.identity() {
                let _: () = jvm
                    .invoke_virtual(&commands, "java/util/Vector", "removeElementAt", "(I)V", (index,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
                    .await?;
                break;
            }
        }

        Ok(())
    }

    async fn set_command_listener(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
    ) -> JvmResult<()> {
        jvm.put_field(&mut this, "commandListener", "Ljavax/microedition/lcdui/CommandListener;", listener)
            .await
    }

    async fn size_changed(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _width: i32, _height: i32) -> JvmResult<()> {
        Ok(())
    }

    async fn set_display(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        display: ClassInstanceRef<Display>,
    ) -> JvmResult<()> {
        let log = format!("javax.microedition.lcdui.Displayable::setDisplay({this:?}, {display:?})");
        tracing::debug!("{log}");

        jvm.put_field(&mut this, "currentDisplay", "Ljavax/microedition/lcdui/Display;", display)
            .await?;
        jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
        jvm.put_field(&mut this, "commandMenuOpen", "Z", false).await?;
        jvm.put_field(&mut this, "commandMenuIndex", "I", -1).await?;

        Ok(())
    }

    async fn request_repaint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if !display.is_null() {
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, -1, -1))
                .await?;
        }

        Ok(())
    }

    async fn decoration_changed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
        jvm.put_field(&mut this, "commandMenuOpen", "Z", false).await?;
        jvm.put_field(&mut this, "commandMenuIndex", "I", -1).await?;

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if !display.is_null() {
            let notification_result: JvmResult<()> = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "notifySizeChanged", "()V", ())
                .await;
            let repaint_result: JvmResult<()> = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                .await;
            notification_result?;
            repaint_result?;
        }

        Ok(())
    }

    async fn get_command_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await
    }

    async fn get_command_at(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
    ) -> JvmResult<ClassInstanceRef<Command>> {
        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    async fn dispatch_command_at(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        let command: ClassInstanceRef<Command> = jvm
            .invoke_virtual(&commands, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await?;
        let listener: ClassInstanceRef<CommandListener> = jvm
            .get_field(&this, "commandListener", "Ljavax/microedition/lcdui/CommandListener;")
            .await?;
        if !listener.is_null() {
            let _: () = jvm
                .invoke_virtual(
                    &listener,
                    "javax/microedition/lcdui/CommandListener",
                    "commandAction",
                    "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                    (command, this),
                )
                .await?;
        }

        Ok(())
    }

    async fn check_item_mutation(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        Ok(())
    }

    async fn item_invalidated(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
        _layout_changed: bool,
    ) -> JvmResult<()> {
        Ok(())
    }

    async fn item_state_changed(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        Err(jvm.exception("java/lang/IllegalStateException", "Item is not owned by a Form").await)
    }

    async fn get_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Displayable::getWidth({this:?})");

        let (width, _) = Self::live_viewport(jvm, context, &this).await?;
        Ok(width)
    }

    async fn get_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Displayable::getHeight({this:?})");

        let (_, height) = Self::live_viewport(jvm, context, &this).await?;
        Ok(height)
    }

    async fn live_viewport(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>) -> JvmResult<(i32, i32)> {
        let display: ClassInstanceRef<Display> = jvm.get_field(this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let (width, height) = if display.is_null() {
            let screen = context.system().platform().screen();
            (screen.width() as i32, screen.height() as i32)
        } else {
            (
                jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "getWidth", "()I", ())
                    .await?,
                jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "getHeight", "()I", ())
                    .await?,
            )
        };
        let content_height: i32 = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getContentHeight",
                "(Ljavax/microedition/lcdui/Displayable;II)I",
                (this.clone(), width, height),
            )
            .await?;

        Ok((width.max(0), content_height))
    }

    async fn notify_size_changed(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        if jvm.get_field::<bool>(&this, "sizeNotifying", "Z").await? {
            return Ok(());
        }

        while jvm.get_field::<bool>(&this, "sizeDirty", "Z").await? {
            let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
            if display.is_null() {
                return Ok(());
            }
            let current: ClassInstanceRef<Displayable> = jvm
                .get_field(&display, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                .await?;
            if current.is_null() || current.identity() != this.identity() {
                return Ok(());
            }

            jvm.put_field(&mut this, "sizeDirty", "Z", false).await?;

            let (width, height) = Self::live_viewport(jvm, context, &this).await?;
            let size_known: bool = jvm.get_field(&this, "sizeKnown", "Z").await?;
            let notified_width: i32 = jvm.get_field(&this, "notifiedWidth", "I").await?;
            let notified_height: i32 = jvm.get_field(&this, "notifiedHeight", "I").await?;
            if size_known && notified_width == width && notified_height == height {
                continue;
            }

            let callback_display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
            if callback_display.is_null() || callback_display.identity() != display.identity() {
                jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
                return Ok(());
            }
            let callback_current: ClassInstanceRef<Displayable> = jvm
                .get_field(&callback_display, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                .await?;
            if callback_current.is_null() || callback_current.identity() != this.identity() {
                jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
                return Ok(());
            }

            jvm.put_field(&mut this, "notifiedWidth", "I", width).await?;
            jvm.put_field(&mut this, "notifiedHeight", "I", height).await?;
            jvm.put_field(&mut this, "sizeKnown", "Z", true).await?;
            jvm.put_field(&mut this, "sizeNotifying", "Z", true).await?;

            let active_display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
            let still_current = if active_display.is_null() || active_display.identity() != display.identity() {
                false
            } else {
                let active_current: ClassInstanceRef<Displayable> = jvm
                    .get_field(&active_display, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                    .await?;
                !active_current.is_null() && active_current.identity() == this.identity()
            };
            if !still_current {
                jvm.put_field(&mut this, "notifiedWidth", "I", notified_width).await?;
                jvm.put_field(&mut this, "notifiedHeight", "I", notified_height).await?;
                jvm.put_field(&mut this, "sizeKnown", "Z", size_known).await?;
                jvm.put_field(&mut this, "sizeNotifying", "Z", false).await?;
                jvm.put_field(&mut this, "sizeDirty", "Z", true).await?;
                return Ok(());
            }

            let result: JvmResult<()> = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "sizeChanged", "(II)V", (width, height))
                .await;
            jvm.put_field(&mut this, "sizeNotifying", "Z", false).await?;
            result?;
        }

        Ok(())
    }

    async fn handle_key_event(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::handleKeyEvent({this:?}, {event_type}, {code})");
        Ok(())
    }

    async fn handle_paint_event(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::handlePaintEvent({this:?}, {graphics:?})");
        Ok(())
    }

    async fn handle_notify_event(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Displayable::handleNotifyEvent({this:?}, {}, {param1}, {param2})",
            r#type
        );
        Ok(())
    }
}
