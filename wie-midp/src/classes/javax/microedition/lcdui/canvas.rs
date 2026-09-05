use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

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

        jvm.put_field(&mut this, "isInFullScreenMode", "Z", mode).await?;

        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if !display.is_null() {
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "setFullscreen", "(Z)V", (mode,))
                .await?;
        }

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
    use alloc::{boxed::Box, vec};
    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
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
                ],
                fields: vec![
                    JavaFieldProto::new("translateX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("translateY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipHeight", "I", FieldAccessFlags::PUBLIC),
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
                jvm.put_field(&mut this, field, "I", value).await?;
            }

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x22aa44,))
                .await?;
            jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 320, 240))
                .await
        }

        async fn size_changed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "sizeChangedCount", "I").await?;
            jvm.put_field(&mut this, "sizeChangedCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastWidth", "I", width).await?;
            jvm.put_field(&mut this, "lastHeight", "I", height).await
        }

        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "pressed", "I", code).await
        }

        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "repeated", "I", code).await
        }

        async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "released", "I", code).await
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
            let count: i32 = jvm.get_field(&this, "sizeChangedCount", "I").await?;
            jvm.put_field(&mut this, "sizeChangedCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastWidth", "I", width).await?;
            jvm.put_field(&mut this, "lastHeight", "I", height).await
        }

        async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "pressed", "I", code).await
        }

        async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "repeated", "I", code).await
        }

        async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "released", "I", code).await
        }
    }

    #[test]
    fn canvas_and_game_canvas_fullscreen_preserve_paint_and_keys() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), [RecordingCanvas::as_proto(), RecordingGameCanvas::as_proto()].into()]),
            |jvm| async move {
                let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                for (class, game_canvas) in [
                    ("javax/microedition/lcdui/TestRecordingCanvas", false),
                    ("javax/microedition/lcdui/TestRecordingGameCanvas", true),
                ] {
                    let canvas: ClassInstanceRef<Canvas> = jvm.new_class(class, "()V", ()).await?.into();
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
                            (command,),
                        )
                        .await?;
                    let decorated_height: i32 = jvm
                        .invoke_virtual(&canvas, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                        .await?;
                    assert!(decorated_height < 240);
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
                        assert_eq!(jvm.get_field::<i32>(&canvas, "sizeChangedCount", "I").await?, callback_count);
                        assert_eq!(jvm.get_field::<i32>(&canvas, "lastHeight", "I").await?, height);
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                            .await?;
                        if !game_canvas {
                            for coordinate in ["translateX", "clipX", "clipY"] {
                                assert_eq!(jvm.get_field::<i32>(&canvas, coordinate, "I").await?, 0);
                            }
                            assert_eq!(jvm.get_field::<i32>(&canvas, "translateY", "I").await? == 0, fullscreen);
                            assert_eq!(jvm.get_field::<i32>(&canvas, "clipWidth", "I").await?, 320);
                            assert_eq!(jvm.get_field::<i32>(&canvas, "clipHeight", "I").await?, height);
                        }
                        let mut graphics = Display::screen_graphics(&jvm, &display).await?;
                        let image_ref = Graphics::image(&jvm, &mut graphics).await?;
                        let image = Image::image(&jvm, &image_ref).await?;
                        let content = image.get_pixel(160, 120);
                        assert_eq!((content.r, content.g, content.b), (0x22, 0xaa, 0x44));
                        for (x, y) in [(0, 0), (319, 239)] {
                            let pixel = image.get_pixel(x, y);
                            assert_eq!((pixel.r, pixel.g, pixel.b) == (0x22, 0xaa, 0x44), fullscreen);
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
                        assert_eq!(jvm.get_field::<i32>(&canvas, field, "I").await?, key);
                    }
                }
                Ok(())
            },
        )
    }
}
