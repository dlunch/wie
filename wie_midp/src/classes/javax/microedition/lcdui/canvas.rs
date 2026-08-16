use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Display, Graphics},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// abstract class javax.microedition.lcdui.Canvas
pub struct Canvas;

impl Canvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Canvas",
            parent_class: Some("javax/microedition/lcdui/Displayable"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new_abstract(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    MethodAccessFlags::PROTECTED | MethodAccessFlags::ABSTRACT,
                ),
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("setFullScreenMode", "(Z)V", Self::set_full_screen_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isDoubleBuffered", "()Z", Self::is_double_buffered, MethodAccessFlags::PUBLIC),
                // wie private methods
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::handle_paint_event,
                    MethodAccessFlags::empty(),
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::repaint({this:?})");

        let display = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let _: () = jvm
            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, -1, -1))
            .await?;

        Ok(())
    }

    async fn repaint_with_area(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::repaint({this:?}, {x}, {y}, {width}, {height})");

        let display = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let _: () = jvm
            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::serviceRepaints({this:?})");

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if !display.is_null() {
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
        }

        Ok(())
    }

    async fn get_game_action(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getGameAction({this:?}, {key})");

        let action = match MIDPKeyCode::from_raw(key) {
            Some(MIDPKeyCode::UP) => 1,    // UP
            Some(MIDPKeyCode::DOWN) => 6,  // DOWN
            Some(MIDPKeyCode::LEFT) => 2,  // LEFT
            Some(MIDPKeyCode::RIGHT) => 5, // RIGHT
            Some(MIDPKeyCode::FIRE) => 8,  // FIRE,
            _ => 0,
        };

        Ok(action)
    }

    async fn key_pressed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyPressed({this:?}, {key})");

        Ok(())
    }

    async fn key_repeated(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyRepeated({this:?}, {key})");

        Ok(())
    }

    async fn key_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyReleased({this:?}, {key})");

        Ok(())
    }

    async fn set_full_screen_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::setFullScreenMode({this:?}, {mode})");

        let previous_mode: bool = jvm.get_field(&this, "isInFullScreenMode", "Z").await?;
        if previous_mode == mode {
            return Ok(());
        }

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if !display.is_null() {
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "setFullscreen", "(Z)V", (mode,))
                .await?;
        }

        jvm.put_field(&mut this, "isInFullScreenMode", "Z", mode).await?;

        Ok(())
    }

    async fn is_double_buffered(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::isDoubleBuffered({this:?})");

        Ok(true)
    }

    async fn handle_key_event(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::handleKeyEvent({this:?}, {event_type}, {code})");

        let event_type = if let Some(event_type) = KeyboardEventType::from_raw(event_type) {
            event_type
        } else {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid keyboard event type").await);
        };

        let _: () = match event_type {
            KeyboardEventType::KeyPressed => {
                jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "keyPressed", "(I)V", (code,))
                    .await
            }
            KeyboardEventType::KeyReleased => {
                jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "keyReleased", "(I)V", (code,))
                    .await
            }
            KeyboardEventType::KeyRepeated => {
                jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "keyRepeated", "(I)V", (code,))
                    .await
            }
            _ => unimplemented!(),
        }?;

        Ok(())
    }

    async fn handle_paint_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::handlePaintEvent({this:?}, {graphics:?})");

        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Canvas",
                "paint",
                "(Ljavax/microedition/lcdui/Graphics;)V",
                (graphics,),
            )
            .await?;

        Ok(())
    }
}
