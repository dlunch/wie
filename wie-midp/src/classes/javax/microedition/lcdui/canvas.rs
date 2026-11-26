use alloc::string::String as RustString;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::{
    javax::microedition::lcdui::{Display, Graphics},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// abstract class javax.microedition.lcdui.Canvas
#[java_class(
    name = "javax/microedition/lcdui/Canvas",
    extends = "javax/microedition/lcdui/Displayable",
    public,
    abstract
)]
pub struct Canvas;

#[java_methods]
impl Canvas {
    #[method(protected)]
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()>;

    #[method(constructor, protected)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    #[method(public)]
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

    #[method(name = "repaint", public)]
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

    #[method(public)]
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

    #[method(public)]
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

    #[method(public)]
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

    #[method(protected)]
    async fn key_pressed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyPressed({this:?}, {key})");

        Ok(())
    }

    #[method(protected)]
    async fn key_repeated(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyRepeated({this:?}, {key})");

        Ok(())
    }

    #[method(protected)]
    async fn key_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::keyReleased({this:?}, {key})");

        Ok(())
    }

    #[method(public)]
    async fn set_full_screen_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, mode: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::setFullScreenMode({this:?}, {mode})");

        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "setFullScreen", "(Z)V", (mode,))
            .await
    }

    #[method(public)]
    async fn is_double_buffered(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::isDoubleBuffered({this:?})");

        Ok(true)
    }

    #[method(public)]
    async fn has_pointer_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::hasPointerEvents({this:?})");

        Ok(false)
    }

    #[method(public)]
    async fn has_pointer_motion_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Canvas::hasPointerMotionEvents({this:?})");

        Ok(false)
    }

    #[method(public)]
    async fn has_repeat_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Canvas::hasRepeatEvents({this:?})");

        Ok(true)
    }

    #[method(public)]
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

    #[method(protected)]
    async fn pointer_pressed(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerPressed({this:?}, {x}, {y})");

        Ok(())
    }

    #[method(protected)]
    async fn pointer_released(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerReleased({this:?}, {x}, {y})");

        Ok(())
    }

    #[method(protected)]
    async fn pointer_dragged(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::pointerDragged({this:?}, {x}, {y})");

        Ok(())
    }

    #[method(protected)]
    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::showNotify({this:?})");

        Ok(())
    }

    #[method(protected)]
    async fn hide_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::hideNotify({this:?})");

        Ok(())
    }

    // Protected so vendor canvases in other Java packages can translate key codes.
    #[method(protected)]
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

    #[method]
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
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{java_class, java_methods};
    use test_utils::run_jvm_test;

    use wie_jvm_support::WieJvmContext;
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::lcdui::{Canvas, Command, Display, Displayable, Graphics, Image},
            net::wie::{KeyboardEventType, MIDPKeyCode},
        },
        get_protos,
    };

    #[java_class(
        name = "javax/microedition/lcdui/TestRecordingCanvas",
        extends = "javax/microedition/lcdui/Canvas",
        public
    )]
    struct RecordingCanvas {
        #[field(public)]
        shown: bool,
        #[field(public)]
        redirectDisplay: ClassInstanceRef<Display>,
        #[field(public)]
        redirect: ClassInstanceRef<Displayable>,
        #[field(public)]
        translateX: i32,
        #[field(public)]
        translateY: i32,
        #[field(public)]
        clipX: i32,
        #[field(public)]
        clipY: i32,
        #[field(public)]
        clipWidth: i32,
        #[field(public)]
        clipHeight: i32,
        #[field(public)]
        paintCount: i32,
        #[field(public)]
        repaintDuringPaint: bool,
        #[field(public)]
        sizeChangedCount: i32,
        #[field(public)]
        lastWidth: i32,
        #[field(public)]
        lastHeight: i32,
        #[field(public)]
        pressed: i32,
        #[field(public)]
        repeated: i32,
        #[field(public)]
        released: i32,
    }
    #[java_class(
        name = "javax/microedition/lcdui/TestRecordingGameCanvas",
        extends = "javax/microedition/lcdui/game/GameCanvas",
        public
    )]
    struct RecordingGameCanvas {
        #[field(public)]
        sizeChangedCount: i32,
        #[field(public)]
        lastWidth: i32,
        #[field(public)]
        lastHeight: i32,
        #[field(public)]
        pressed: i32,
        #[field(public)]
        repeated: i32,
        #[field(public)]
        released: i32,
    }

    #[java_methods]
    impl RecordingCanvas {
        #[method(constructor, public)]
        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        #[method(protected)]
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

        #[method(protected)]
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

        #[method(protected)]
        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "pressed", "I", code)
                .await
        }

        #[method(protected)]
        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "repeated", "I", code)
                .await
        }

        #[method(protected)]
        async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "released", "I", code)
                .await
        }

        #[method(protected)]
        async fn show_notify(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingCanvas", "shown", "Z", true)
                .await
        }

        #[method(protected)]
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

    #[java_methods]
    impl RecordingGameCanvas {
        #[method(constructor, public)]
        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/game/GameCanvas", "<init>", "(Z)V", (false,))
                .await
        }

        #[method(protected)]
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

        #[method(protected)]
        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "pressed", "I", code)
                .await
        }

        #[method(protected)]
        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "javax/microedition/lcdui/TestRecordingGameCanvas", "repeated", "I", code)
                .await
        }

        #[method(protected)]
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
