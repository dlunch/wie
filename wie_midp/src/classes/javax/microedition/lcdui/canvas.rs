use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::ClassAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::Graphics,
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
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("repaint", "()V", Self::repaint, Default::default()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, Default::default()),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, Default::default()),
                JavaMethodProto::new_abstract("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Default::default()),
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, Default::default()),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, Default::default()),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, Default::default()),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, Default::default()),
                JavaMethodProto::new("setFullScreenMode", "(Z)V", Self::set_full_screen_mode, Default::default()),
                // wie private methods
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, Default::default()),
                JavaMethodProto::new(
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::handle_paint_event,
                    Default::default(),
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::<init>({:?})", &this);

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::repaint({:?})", &this);

        jvm.invoke_virtual(&this, "repaint", "(IIII)V", (0, 0, 0, 0)).await
    }

    async fn repaint_with_area(
        _jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub javax.microedition.lcdui.Canvas::repaint({:?}, {}, {}, {}, {})",
            &this,
            x,
            y,
            width,
            height
        );

        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }

    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::serviceRepaints({:?})", &this);

        jvm.invoke_virtual(&this, "repaint", "(IIII)V", (0, 0, 0, 0)).await
    }

    async fn get_game_action(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getGameAction({:?}, {})", &this, key);

        let action = match MIDPKeyCode::from_raw(key) {
            MIDPKeyCode::UP => 1,    // UP
            MIDPKeyCode::DOWN => 6,  // DOWN
            MIDPKeyCode::LEFT => 2,  // LEFT
            MIDPKeyCode::RIGHT => 5, // RIGHT
            MIDPKeyCode::FIRE => 8,  // FIRE,
            _ => 0,
        };

        Ok(action)
    }

    async fn key_pressed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyPressed({:?}, {})", &this, key);

        Ok(())
    }

    async fn key_repeated(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyRepeated({:?}, {})", &this, key);

        Ok(())
    }

    async fn key_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyReleased({:?}, {})", &this, key);

        Ok(())
    }

    async fn set_full_screen_mode(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, mode: bool) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::setFullScreenMode({this:?}, {mode})");

        Ok(())
    }

    async fn handle_key_event(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::handleKeyEvent({this:?}, {event_type}, {code})");

        let event_type = KeyboardEventType::from_raw(event_type);

        let _: () = match event_type {
            KeyboardEventType::KeyPressed => jvm.invoke_virtual(&this, "keyPressed", "(I)V", (code,)).await,
            KeyboardEventType::KeyReleased => jvm.invoke_virtual(&this, "keyReleased", "(I)V", (code,)).await,
            KeyboardEventType::KeyRepeated => jvm.invoke_virtual(&this, "keyRepeated", "(I)V", (code,)).await,
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
            .invoke_virtual(&this, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
            .await?;

        Ok(())
    }
}
