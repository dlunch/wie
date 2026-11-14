use alloc::{string::String as RustString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

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
                JavaMethodProto::new("getKeyCode", "(I)I", Self::get_key_code, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("setFullScreenMode", "(Z)V", Self::set_full_screen_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isDoubleBuffered", "()Z", Self::is_double_buffered, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasPointerEvents", "()Z", Self::has_pointer_events, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "hasPointerMotionEvents",
                    "()Z",
                    Self::has_pointer_motion_events,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("hasRepeatEvents", "()Z", Self::has_repeat_events, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getKeyName", "(I)Ljava/lang/String;", Self::get_key_name, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("pointerPressed", "(II)V", Self::pointer_pressed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("pointerReleased", "(II)V", Self::pointer_released, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("pointerDragged", "(II)V", Self::pointer_dragged, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("showNotify", "()V", Self::show_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("hideNotify", "()V", Self::hide_notify, MethodAccessFlags::PROTECTED),
                // Protected so vendor canvases in other Java packages can translate key codes.
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::PROTECTED),
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

        let display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if display.is_null() {
            return Ok(());
        }

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

        if width <= 0 || height <= 0 {
            return Ok(());
        }
        let display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if display.is_null() {
            return Ok(());
        }

        let _: () = jvm
            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::serviceRepaints({this:?})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if !display.is_null() {
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "serviceRepaints", "()V", ())
                .await?;
        }

        Ok(())
    }

    async fn get_game_action(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getGameAction({this:?}, {key})");

        let action = match MIDPKeyCode::from_raw(key) {
            Some(MIDPKeyCode::UP) => 1,        // UP
            Some(MIDPKeyCode::DOWN) => 6,      // DOWN
            Some(MIDPKeyCode::LEFT) => 2,      // LEFT
            Some(MIDPKeyCode::RIGHT) => 5,     // RIGHT
            Some(MIDPKeyCode::FIRE) => 8,      // FIRE
            Some(MIDPKeyCode::KEY_NUM1) => 9,  // GAME_A
            Some(MIDPKeyCode::KEY_NUM3) => 10, // GAME_B
            Some(MIDPKeyCode::KEY_NUM7) => 11, // GAME_C
            Some(MIDPKeyCode::KEY_NUM9) => 12, // GAME_D
            _ => 0,
        };

        Ok(action)
    }

    async fn get_key_code(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, action: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getKeyCode({this:?}, {action})");

        let key = match action {
            1 => MIDPKeyCode::UP,
            6 => MIDPKeyCode::DOWN,
            2 => MIDPKeyCode::LEFT,
            5 => MIDPKeyCode::RIGHT,
            8 => MIDPKeyCode::FIRE,
            9 => MIDPKeyCode::KEY_NUM1,  // GAME_A
            10 => MIDPKeyCode::KEY_NUM3, // GAME_B
            11 => MIDPKeyCode::KEY_NUM7, // GAME_C
            12 => MIDPKeyCode::KEY_NUM9, // GAME_D
            _ => return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid game action").await),
        };

        Ok(key as i32)
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

    async fn set_full_screen_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, mode: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::setFullScreenMode({this:?}, {mode})");

        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "setFullScreen", "(Z)V", (mode,))
            .await
    }

    async fn is_double_buffered(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::isDoubleBuffered({this:?})");

        Ok(true)
    }

    async fn has_pointer_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::hasPointerEvents({this:?})");

        Ok(false)
    }

    async fn has_pointer_motion_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::hasPointerMotionEvents({this:?})");

        Ok(false)
    }

    async fn has_repeat_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Canvas::hasRepeatEvents({this:?})");

        Ok(true)
    }

    async fn get_key_name(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getKeyName({this:?}, {key_code})");

        let name = match MIDPKeyCode::from_raw(key_code) {
            Some(MIDPKeyCode::UP) => "UP".into(),
            Some(MIDPKeyCode::DOWN) => "DOWN".into(),
            Some(MIDPKeyCode::LEFT) => "LEFT".into(),
            Some(MIDPKeyCode::RIGHT) => "RIGHT".into(),
            Some(MIDPKeyCode::FIRE) => "SELECT".into(),
            Some(MIDPKeyCode::LEFT_SOFT_KEY) => "SOFT1".into(),
            Some(MIDPKeyCode::RIGHT_SOFT_KEY) => "SOFT2".into(),
            Some(MIDPKeyCode::CLEAR) => "CLEAR".into(),
            Some(MIDPKeyCode::CALL) => "SEND".into(),
            Some(MIDPKeyCode::HANGUP) => "END".into(),
            Some(MIDPKeyCode::VOLUME_UP) => "VOLUME UP".into(),
            Some(MIDPKeyCode::VOLUME_DOWN) => "VOLUME DOWN".into(),
            // The remaining key codes are the ASCII codes of the keypad characters.
            Some(_) => RustString::from(key_code as u8 as char),
            None => return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid key code").await),
        };

        Ok(JavaLangString::from_rust_string(jvm, &name).await?.into())
    }

    async fn pointer_pressed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerPressed({this:?}, {x}, {y})");

        Ok(())
    }

    async fn pointer_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerReleased({this:?}, {x}, {y})");

        Ok(())
    }

    async fn pointer_dragged(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerDragged({this:?}, {x}, {y})");

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::showNotify({this:?})");

        Ok(())
    }

    async fn hide_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::hideNotify({this:?})");

        Ok(())
    }

    async fn handle_key_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        event_type: i32,
        code: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::handleKeyEvent({this:?}, {event_type}, {code})");

        let event_type = if let Some(event_type) = KeyboardEventType::from_raw(event_type) {
            event_type
        } else {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid keyboard event type").await);
        };

        let action: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "getGameAction", "(I)I", (code,))
            .await?;
        if action != 0 {
            let held: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Displayable", "heldGameKeys", "I").await?;
            match event_type {
                KeyboardEventType::KeyPressed => {
                    let pressed: i32 = jvm
                        .get_field(&this, "javax/microedition/lcdui/Displayable", "pressedGameKeys", "I")
                        .await?;
                    jvm.put_field(
                        &mut this,
                        "javax/microedition/lcdui/Displayable",
                        "heldGameKeys",
                        "I",
                        held | (1 << action),
                    )
                    .await?;
                    jvm.put_field(
                        &mut this,
                        "javax/microedition/lcdui/Displayable",
                        "pressedGameKeys",
                        "I",
                        pressed | (1 << action),
                    )
                    .await?;
                }
                KeyboardEventType::KeyReleased => {
                    jvm.put_field(
                        &mut this,
                        "javax/microedition/lcdui/Displayable",
                        "heldGameKeys",
                        "I",
                        held & !(1 << action),
                    )
                    .await?;
                }
                _ => {}
            }
        }

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
            KeyboardEventType::KeyTyped => Ok(()),
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

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};
    use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::lcdui::{Canvas, Command, Display, Graphics, Image},
            net::wie::{KeyboardEventType, MIDPKeyCode},
        },
        get_protos,
    };

    struct RecordingCanvas;
    struct RecordingGameCanvas;

    impl RecordingCanvas {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingCanvas",
                parent_class: Some("javax/microedition/lcdui/Canvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        Self::paint,
                        MethodAccessFlags::PROTECTED,
                    ),
                    JavaMethodProto::new("sizeChanged", "(II)V", Self::size_changed, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("showNotify", "()V", Self::show_notify, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("hideNotify", "()V", Self::hide_notify, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![
                    JavaFieldProto::new("shown", "Z", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("redirectDisplay", "Ljavax/microedition/lcdui/Display;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("redirect", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("translateX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("translateY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("paintCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintDuringPaint", "Z", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("sizeChangedCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("pressed", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repeated", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("released", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        async fn paint(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            graphics: ClassInstanceRef<Graphics>,
        ) -> JvmResult<()> {
            let count: i32 = jvm
                .get_field(&this, "javax/microedition/lcdui/TestRecordingCanvas", "paintCount", "I")
                .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "paintCount", "I", count + 1)
                .await?;
            for (field, method) in [
                ("translateX", "getTranslateX"),
                ("translateY", "getTranslateY"),
                ("clipX", "getClipX"),
                ("clipY", "getClipY"),
                ("clipWidth", "getClipWidth"),
                ("clipHeight", "getClipHeight"),
            ] {
                let value: i32 = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", method, "()I", ())
                    .await?;
                jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", field, "I", value)
                    .await?;
            }

            if jvm
                .get_field::<bool>(&this, "javax/microedition/lcdui/TestRecordingCanvas", "repaintDuringPaint", "Z")
                .await?
            {
                jvm.put_field(
                    &mut this,
                    "javax/microedition/lcdui/TestRecordingCanvas",
                    "repaintDuringPaint",
                    "Z",
                    false,
                )
                .await?;
                jvm.invoke_virtual::<_, ()>(&this, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (4, 5, 6, 7))
                    .await?;
                // Servicing repaints from inside paint must leave the request above for the next cycle.
                jvm.invoke_virtual::<_, ()>(&this, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                    .await?;
            }

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x22aa44,))
                .await?;
            jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 320, 240))
                .await
        }

        async fn size_changed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
            let count: i32 = jvm
                .get_field(&this, "javax/microedition/lcdui/TestRecordingCanvas", "sizeChangedCount", "I")
                .await?;
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/TestRecordingCanvas",
                "sizeChangedCount",
                "I",
                count + 1,
            )
            .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "lastWidth", "I", width)
                .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "lastHeight", "I", height)
                .await
        }

        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "pressed", "I", code)
                .await
        }

        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "repeated", "I", code)
                .await
        }

        async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "released", "I", code)
                .await
        }

        async fn show_notify(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "shown", "Z", true)
                .await
        }

        async fn hide_notify(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "shown", "Z", false)
                .await?;

            let redirect: ClassInstanceRef<Canvas> = jvm
                .get_field(
                    &this,
                    "javax/microedition/lcdui/TestRecordingCanvas",
                    "redirect",
                    "Ljavax/microedition/lcdui/Displayable;",
                )
                .await?;
            if redirect.is_null() {
                return Ok(());
            }
            let display: ClassInstanceRef<Display> = jvm
                .get_field(
                    &this,
                    "javax/microedition/lcdui/TestRecordingCanvas",
                    "redirectDisplay",
                    "Ljavax/microedition/lcdui/Display;",
                )
                .await?;
            jvm.invoke_virtual(
                &display,
                "javax/microedition/lcdui/Display",
                "setCurrent",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (redirect,),
            )
            .await
        }
    }

    impl RecordingGameCanvas {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingGameCanvas",
                parent_class: Some("javax/microedition/lcdui/game/GameCanvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("sizeChanged", "(II)V", Self::size_changed, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![
                    JavaFieldProto::new("sizeChangedCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("pressed", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repeated", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("released", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/game/GameCanvas", "<init>", "(Z)V", (false,))
                .await
        }

        async fn size_changed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
            let count: i32 = jvm
                .get_field(&this, "javax/microedition/lcdui/TestRecordingGameCanvas", "sizeChangedCount", "I")
                .await?;
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/TestRecordingGameCanvas",
                "sizeChangedCount",
                "I",
                count + 1,
            )
            .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "lastWidth", "I", width)
                .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "lastHeight", "I", height)
                .await
        }

        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "pressed", "I", code)
                .await
        }

        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "repeated", "I", code)
                .await
        }

        async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "released", "I", code)
                .await
        }
    }

    #[test]
    fn canvas_and_game_canvas_fullscreen_preserve_paint_and_keys() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), [RecordingCanvas::as_proto(), RecordingGameCanvas::as_proto()].into()]),
            |jvm| async move {
                let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                let mut recording_canvas: ClassInstanceRef<Canvas> = None.into();
                for (class, game_canvas, skvm) in [
                    ("javax/microedition/lcdui/TestRecordingCanvas", false, false),
                    ("javax/microedition/lcdui/TestRecordingGameCanvas", true, false),
                    ("javax/microedition/lcdui/TestRecordingCanvas", false, true),
                    ("javax/microedition/lcdui/TestRecordingGameCanvas", true, true),
                ] {
                    if skvm {
                        let key = JavaLangString::from_rust_string(&jvm, "m.SK_VM").await?;
                        let value = JavaLangString::from_rust_string(&jvm, "10").await?;
                        let _: ClassInstanceRef<()> = jvm
                            .invoke_static(
                                "java/lang/System",
                                "setProperty",
                                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                                (key, value),
                            )
                            .await?;
                    }
                    let mut canvas: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                            .await?,
                        if skvm { 224 } else { 240 }
                    );
                    for (action, expected_key) in [
                        (1, MIDPKeyCode::UP),
                        (6, MIDPKeyCode::DOWN),
                        (2, MIDPKeyCode::LEFT),
                        (5, MIDPKeyCode::RIGHT),
                        (8, MIDPKeyCode::FIRE),
                        (9, MIDPKeyCode::KEY_NUM1),
                        (10, MIDPKeyCode::KEY_NUM3),
                        (11, MIDPKeyCode::KEY_NUM7),
                        (12, MIDPKeyCode::KEY_NUM9),
                    ] {
                        let key: i32 = jvm
                            .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "getKeyCode", "(I)I", (action,))
                            .await?;
                        assert_eq!(key, expected_key as i32);
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", "getGameAction", "(I)I", (key,))
                                .await?,
                            action
                        );
                    }
                    for action in [-1, 0, 3, 4, 7, 13] {
                        let result = jvm
                            .invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", "getKeyCode", "(I)I", (action,))
                            .await;
                        let Err(JavaError::JavaException(exception)) = result else {
                            panic!("Expected IllegalArgumentException for action {action}");
                        };
                        assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                    }
                    let title = JavaLangString::from_rust_string(&jvm, "Canvas title").await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &canvas,
                            "javax/microedition/lcdui/Displayable",
                            "setTitle",
                            "(Ljava/lang/String;)V",
                            (title,),
                        )
                        .await?;
                    let label = JavaLangString::from_rust_string(&jvm, "Select").await?;
                    let command: ClassInstanceRef<Command> = jvm
                        .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 4, 0))
                        .await?
                        .into();
                    let _: () = jvm
                        .invoke_virtual(
                            &canvas,
                            "javax/microedition/lcdui/Displayable",
                            "addCommand",
                            "(Ljavax/microedition/lcdui/Command;)V",
                            (command.clone(),),
                        )
                        .await?;
                    let decorated_height: i32 = jvm
                        .invoke_virtual(&canvas, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                        .await?;
                    let command_height = if skvm { 16 } else { 18 };
                    assert_eq!(decorated_height, 240 - 16 - command_height);
                    if game_canvas {
                        let graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_virtual(
                                &canvas,
                                "javax/microedition/lcdui/game/GameCanvas",
                                "getGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x22aa44,))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 320, 240))
                            .await?;
                    }
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "javax/microedition/lcdui/Display",
                            "setCurrent",
                            "(Ljavax/microedition/lcdui/Displayable;)V",
                            (canvas.clone(),),
                        )
                        .await?;
                    // The recording canvas is shown when it becomes current and hidden once the game canvas replaces it.
                    if !game_canvas {
                        recording_canvas = canvas.clone();
                    }
                    assert_eq!(
                        jvm.get_field::<bool>(&recording_canvas, "javax/microedition/lcdui/TestRecordingCanvas", "shown", "Z")
                            .await?,
                        !game_canvas
                    );
                    for (fullscreen, callback_count, event_type, field, key) in [
                        (false, 1, KeyboardEventType::KeyPressed, "pressed", MIDPKeyCode::KEY_NUM1),
                        (true, 2, KeyboardEventType::KeyRepeated, "repeated", MIDPKeyCode::KEY_NUM2),
                        (false, 3, KeyboardEventType::KeyReleased, "released", MIDPKeyCode::KEY_NUM3),
                    ] {
                        let _: () = jvm
                            .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "setFullScreenMode", "(Z)V", (fullscreen,))
                            .await?;
                        let key = key as i32;
                        let height = if fullscreen { 240 } else { decorated_height };
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                                .await?,
                            height
                        );
                        assert_eq!(jvm.get_field::<i32>(&canvas, class, "sizeChangedCount", "I").await?, callback_count);
                        assert_eq!(jvm.get_field::<i32>(&canvas, class, "lastHeight", "I").await?, height);
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                            .await?;
                        if !game_canvas {
                            for coordinate in ["translateX", "clipX", "clipY"] {
                                assert_eq!(jvm.get_field::<i32>(&canvas, class, coordinate, "I").await?, 0);
                            }
                            assert_eq!(
                                jvm.get_field::<i32>(&canvas, "javax/microedition/lcdui/TestRecordingCanvas", "translateY", "I")
                                    .await?
                                    == 0,
                                fullscreen
                            );
                            assert_eq!(
                                jvm.get_field::<i32>(&canvas, "javax/microedition/lcdui/TestRecordingCanvas", "clipWidth", "I")
                                    .await?,
                                320
                            );
                            assert_eq!(
                                jvm.get_field::<i32>(&canvas, "javax/microedition/lcdui/TestRecordingCanvas", "clipHeight", "I")
                                    .await?,
                                height
                            );
                        }
                        let mut graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_static(
                                "javax/microedition/lcdui/Display",
                                "getScreenGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let image_ref = Graphics::image(&jvm, &mut graphics).await?;
                        let image = Image::image(&jvm, &image_ref).await?;
                        let content = image.get_pixel(160, 120);
                        assert_eq!((content.r, content.g, content.b), (0x22, 0xaa, 0x44));
                        for (x, y) in [(0, 0), (319, 239)] {
                            let pixel = image.get_pixel(x, y);
                            assert_eq!((pixel.r, pixel.g, pixel.b) == (0x22, 0xaa, 0x44), fullscreen);
                        }
                        if !game_canvas && callback_count == 1 {
                            for bounds in [(10, 20, 30, 40), (25, 5, 40, 25)] {
                                jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", bounds)
                                    .await?;
                            }
                            jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                                .await?;
                            for (field, expected) in [("clipX", 10), ("clipY", 5), ("clipWidth", 55), ("clipHeight", 55)] {
                                assert_eq!(jvm.get_field::<i32>(&canvas, class, field, "I").await?, expected);
                            }

                            jvm.put_field(&mut canvas, class, "repaintDuringPaint", "Z", true).await?;
                            jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (-5, -6, 10, 12))
                                .await?;
                            for expected in [[0, 0, 5, 6], [4, 5, 6, 7]] {
                                jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                                    .await?;
                                for (field, value) in ["clipX", "clipY", "clipWidth", "clipHeight"].into_iter().zip(expected) {
                                    assert_eq!(jvm.get_field::<i32>(&canvas, class, field, "I").await?, value);
                                }
                            }
                            let count: i32 = jvm.get_field(&canvas, class, "paintCount", "I").await?;
                            for bounds in [(0, 0, 0, 10), (0, 0, -1, -1), (400, 0, 10, 10)] {
                                jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", bounds)
                                    .await?;
                            }
                            jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                                .await?;
                            assert_eq!(jvm.get_field::<i32>(&canvas, class, "paintCount", "I").await?, count);
                            jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "()V", ())
                                .await?;
                            jvm.invoke_virtual::<_, ()>(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                                .await?;
                            assert_eq!(jvm.get_field::<i32>(&canvas, class, "clipWidth", "I").await?, 320);
                            assert_eq!(jvm.get_field::<i32>(&canvas, class, "clipHeight", "I").await?, height);
                        }
                        let _: () = jvm
                            .invoke_virtual(
                                &display,
                                "javax/microedition/lcdui/Display",
                                "handleKeyEvent",
                                "(II)V",
                                (event_type as i32, key),
                            )
                            .await?;
                        assert_eq!(jvm.get_field::<i32>(&canvas, class, field, "I").await?, key);
                    }
                    if game_canvas {
                        // KEY_NUM1 (GAME_A) is still held; it is then released and KEY_NUM3 (GAME_B) tapped between two polls.
                        let tap = [
                            (KeyboardEventType::KeyReleased as i32, MIDPKeyCode::KEY_NUM1 as i32),
                            (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::KEY_NUM3 as i32),
                            (KeyboardEventType::KeyReleased as i32, MIDPKeyCode::KEY_NUM3 as i32),
                        ];
                        let mut states = Vec::new();
                        for events in [&[][..], &tap[..], &[][..]] {
                            for &event in events {
                                let _: () = jvm
                                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handleKeyEvent", "(II)V", event)
                                    .await?;
                            }
                            let state: i32 = jvm
                                .invoke_virtual(&canvas, "javax/microedition/lcdui/game/GameCanvas", "getKeyStates", "()I", ())
                                .await?;
                            states.push(state);
                        }
                        assert_eq!(states, [1 << 9, 1 << 10, 0]);

                        // A key held while another screen takes over reads as unpressed once this canvas is shown again.
                        let _: () = jvm
                            .invoke_virtual(
                                &display,
                                "javax/microedition/lcdui/Display",
                                "handleKeyEvent",
                                "(II)V",
                                (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::KEY_NUM1 as i32),
                            )
                            .await?;
                        for shown in [recording_canvas.clone(), canvas.clone()] {
                            let _: () = jvm
                                .invoke_virtual(
                                    &display,
                                    "javax/microedition/lcdui/Display",
                                    "setCurrent",
                                    "(Ljavax/microedition/lcdui/Displayable;)V",
                                    (shown,),
                                )
                                .await?;
                        }
                        let state: i32 = jvm
                            .invoke_virtual(&canvas, "javax/microedition/lcdui/game/GameCanvas", "getKeyStates", "()I", ())
                            .await?;
                        assert_eq!(state, 0);

                        // With a second command the left soft key opens the command menu, which then consumes the UP release.
                        let label = JavaLangString::from_rust_string(&jvm, "More").await?;
                        let extra: ClassInstanceRef<Command> = jvm
                            .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 4, 1))
                            .await?
                            .into();
                        let _: () = jvm
                            .invoke_virtual(
                                &canvas,
                                "javax/microedition/lcdui/Displayable",
                                "addCommand",
                                "(Ljavax/microedition/lcdui/Command;)V",
                                (extra.clone(),),
                            )
                            .await?;
                        for event in [
                            (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::UP as i32),
                            (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::LEFT_SOFT_KEY as i32),
                            (KeyboardEventType::KeyReleased as i32, MIDPKeyCode::UP as i32),
                            (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::RIGHT_SOFT_KEY as i32),
                        ] {
                            let _: () = jvm
                                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handleKeyEvent", "(II)V", event)
                                .await?;
                        }
                        let state: i32 = jvm
                            .invoke_virtual(&canvas, "javax/microedition/lcdui/game/GameCanvas", "getKeyStates", "()I", ())
                            .await?;
                        assert_eq!(state, 0);
                        let _: () = jvm
                            .invoke_virtual(
                                &canvas,
                                "javax/microedition/lcdui/Displayable",
                                "removeCommand",
                                "(Ljavax/microedition/lcdui/Command;)V",
                                (extra,),
                            )
                            .await?;
                    }
                    let _: () = jvm
                        .invoke_virtual(
                            &canvas,
                            "javax/microedition/lcdui/Displayable",
                            "removeCommand",
                            "(Ljavax/microedition/lcdui/Command;)V",
                            (command,),
                        )
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &canvas,
                            "javax/microedition/lcdui/Displayable",
                            "setTitle",
                            "(Ljava/lang/String;)V",
                            (ClassInstanceRef::<()>::new(None),),
                        )
                        .await?;
                    let height = if skvm { 224 } else { 240 };
                    assert_eq!(
                        jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                            .await?,
                        height
                    );
                    assert_eq!(jvm.get_field::<i32>(&canvas, class, "lastHeight", "I").await?, height);
                    let _: () = jvm
                        .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (0, 0, 320, 240))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                        .await?;
                    let mut graphics: ClassInstanceRef<Graphics> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Display",
                            "getScreenGraphics",
                            "()Ljavax/microedition/lcdui/Graphics;",
                            (),
                        )
                        .await?;
                    let image_ref = Graphics::image(&jvm, &mut graphics).await?;
                    let bottom = Image::image(&jvm, &image_ref).await?.get_pixel(319, 239);
                    assert_eq!((bottom.r, bottom.g, bottom.b), (0x22, 0xaa, 0x44));
                }

                // The first canvas switches to `redirect` from hideNotify, which supersedes the switch to `superseded`.
                let class = "javax/microedition/lcdui/TestRecordingCanvas";
                let mut first: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
                let superseded: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
                let redirect: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
                jvm.put_field(
                    &mut first,
                    class,
                    "redirectDisplay",
                    "Ljavax/microedition/lcdui/Display;",
                    display.clone(),
                )
                .await?;
                jvm.put_field(&mut first, class, "redirect", "Ljavax/microedition/lcdui/Displayable;", redirect.clone())
                    .await?;
                for shown in [first, superseded.clone()] {
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "javax/microedition/lcdui/Display",
                            "setCurrent",
                            "(Ljavax/microedition/lcdui/Displayable;)V",
                            (shown,),
                        )
                        .await?;
                }
                let current: ClassInstanceRef<Canvas> = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "getCurrent",
                        "()Ljavax/microedition/lcdui/Displayable;",
                        (),
                    )
                    .await?;
                assert_eq!(current.identity(), redirect.identity());
                assert!(jvm.get_field::<bool>(&redirect, class, "shown", "Z").await?);
                assert!(!jvm.get_field::<bool>(&superseded, class, "shown", "Z").await?);

                // A hideNotify that switches to the screen already being shown must not cancel that screen's activation.
                let mut hidden: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
                jvm.put_field(
                    &mut hidden,
                    class,
                    "redirectDisplay",
                    "Ljavax/microedition/lcdui/Display;",
                    display.clone(),
                )
                .await?;
                jvm.put_field(
                    &mut hidden,
                    class,
                    "redirect",
                    "Ljavax/microedition/lcdui/Displayable;",
                    superseded.clone(),
                )
                .await?;
                for shown in [hidden, superseded.clone()] {
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "javax/microedition/lcdui/Display",
                            "setCurrent",
                            "(Ljavax/microedition/lcdui/Displayable;)V",
                            (shown,),
                        )
                        .await?;
                }
                assert!(jvm.get_field::<bool>(&superseded, class, "shown", "Z").await?);
                assert!(
                    jvm.invoke_virtual::<_, bool>(&superseded, "javax/microedition/lcdui/Displayable", "isShown", "()Z", ())
                        .await?
                );
                Ok(())
            },
        )
    }
}
