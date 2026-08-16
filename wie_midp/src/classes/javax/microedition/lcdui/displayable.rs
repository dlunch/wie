use alloc::{format, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Command, CommandListener, Display, Graphics};

// class javax.microedition.lcdui.Displayable
pub struct Displayable;

impl Displayable {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Displayable",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::add_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    Self::set_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                // wie private methods...
                JavaMethodProto::new(
                    "setDisplay",
                    "(Ljavax/microedition/lcdui/Display;)V",
                    Self::set_display,
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
                JavaFieldProto::new("currentDisplay", "Ljavax/microedition/lcdui/Display;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("isInFullScreenMode", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Displayable::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn add_command(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Displayable::addCommand({this:?}, {command:?})");

        Ok(())
    }

    async fn set_command_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Displayable::setCommandListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn set_display(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        display: ClassInstanceRef<Display>,
    ) -> JvmResult<()> {
        // tracing hates variable named `display`..
        let log = format!("javax.microedition.lcdui.Displayable::setDisplay({this:?}, {display:?})");
        tracing::debug!("{log}");

        jvm.put_field(&mut this, "currentDisplay", "Ljavax/microedition/lcdui/Display;", display)
            .await?;

        Ok(())
    }

    async fn get_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Displayable::getWidth({this:?})");

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let width = if display.is_null() {
            context.system().platform().screen().width() as i32
        } else {
            jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "getWidth", "()I", ())
                .await?
        };

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Displayable::getHeight({this:?})");

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let height = if display.is_null() {
            context.system().platform().screen().height() as i32
        } else {
            jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "getHeight", "()I", ())
                .await?
        };

        Ok(height)
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
            r#type,
        );

        Ok(())
    }
}
