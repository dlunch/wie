use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use test_utils::run_jvm_test;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::{
    javax::microedition::lcdui::{Display, Graphics},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

struct RecordingCanvas;

impl RecordingCanvas {
    fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "test/RecordingCanvas",
            parent_class: Some("com/nokia/mid/ui/FullCanvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::paint,
                    MethodAccessFlags::PROTECTED,
                ),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("painted", "Z", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("pressed", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("repeated", "I", FieldAccessFlags::PUBLIC),
                JavaFieldProto::new("released", "I", FieldAccessFlags::PUBLIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        jvm.invoke_special(&this, "com/nokia/mid/ui/FullCanvas", "<init>", "()V", ()).await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        for (method, expected) in [("getClipWidth", 320), ("getClipHeight", 240)] {
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", method, "()I", ())
                    .await?,
                expected
            );
        }
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 320, 240))
            .await?;
        jvm.put_field(&mut this, "test/RecordingCanvas", "painted", "Z", true).await
    }

    async fn key_pressed(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "test/RecordingCanvas", "pressed", "I", code).await
    }

    async fn key_repeated(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "test/RecordingCanvas", "repeated", "I", code).await
    }

    async fn key_released(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, code: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "test/RecordingCanvas", "released", "I", code).await
    }
}

#[test]
fn full_canvas_renders_and_translates_keys() {
    run_jvm_test(
        Box::new([
            wie_midp::get_protos().into(),
            wie_midp_vendor::get_protos().into(),
            [RecordingCanvas::as_proto()].into(),
        ]),
        |jvm| async move {
            let canvas = jvm.new_class("test/RecordingCanvas", "()V", ()).await?;
            let title = JavaLangString::from_rust_string(&jvm, "Fullscreen canvas").await?;
            let _: () = jvm
                .invoke_virtual(
                    &canvas,
                    "javax/microedition/lcdui/Displayable",
                    "setTitle",
                    "(Ljava/lang/String;)V",
                    (title,),
                )
                .await?;
            for (method, expected) in [("getWidth", 320), ("getHeight", 240)] {
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", method, "()I", ())
                        .await?,
                    expected
                );
            }

            let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (canvas.clone(),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                .await?;
            assert!(jvm.get_field::<bool>(&canvas, "test/RecordingCanvas", "painted", "Z").await?);

            for (name, input, expected) in [
                ("KEY_UP_ARROW", MIDPKeyCode::UP, -1),
                ("KEY_DOWN_ARROW", MIDPKeyCode::DOWN, -2),
                ("KEY_LEFT_ARROW", MIDPKeyCode::LEFT, -3),
                ("KEY_RIGHT_ARROW", MIDPKeyCode::RIGHT, -4),
                ("KEY_SOFTKEY3", MIDPKeyCode::FIRE, -5),
                ("KEY_SOFTKEY1", MIDPKeyCode::LEFT_SOFT_KEY, -6),
                ("KEY_SOFTKEY2", MIDPKeyCode::RIGHT_SOFT_KEY, -7),
                ("KEY_SEND", MIDPKeyCode::CALL, -10),
                ("KEY_END", MIDPKeyCode::HANGUP, -11),
            ] {
                assert_eq!(jvm.get_static_field::<i32>("com/nokia/mid/ui/FullCanvas", name, "I").await?, expected);
                let input = input as i32;
                for (event_type, field) in [
                    (KeyboardEventType::KeyPressed, "pressed"),
                    (KeyboardEventType::KeyRepeated, "repeated"),
                    (KeyboardEventType::KeyReleased, "released"),
                ] {
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "javax/microedition/lcdui/Display",
                            "handleKeyEvent",
                            "(II)V",
                            (event_type as i32, input),
                        )
                        .await?;
                    assert_eq!(jvm.get_field::<i32>(&canvas, "test/RecordingCanvas", field, "I").await?, expected);
                }
            }
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "handleKeyEvent",
                    "(II)V",
                    (KeyboardEventType::KeyPressed as i32, 53),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&canvas, "test/RecordingCanvas", "pressed", "I").await?, 53);

            for (action, code) in [(1, -1), (6, -2), (2, -3), (5, -4), (8, -5), (9, 49), (10, 51), (11, 55), (12, 57)] {
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", "getKeyCode", "(I)I", (action,))
                        .await?,
                    code
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", "getGameAction", "(I)I", (code,))
                        .await?,
                    action
                );
            }
            for code in [-6, -7, -10, -11] {
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "javax/microedition/lcdui/Canvas", "getGameAction", "(I)I", (code,))
                        .await?,
                    0
                );
            }

            for (method, descriptor) in [
                ("addCommand", "(Ljavax/microedition/lcdui/Command;)V"),
                ("setCommandListener", "(Ljavax/microedition/lcdui/CommandListener;)V"),
            ] {
                let result = jvm
                    .invoke_virtual::<_, ()>(
                        &canvas,
                        "javax/microedition/lcdui/Displayable",
                        method,
                        descriptor,
                        (ClassInstanceRef::<()>::new(None),),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("Expected IllegalStateException from {method}");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));
            }
            Ok(())
        },
    )
    .unwrap();
}
