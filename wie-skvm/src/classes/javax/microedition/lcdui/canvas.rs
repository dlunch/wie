use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::MethodAccessFlags;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Canvas as MidpCanvas;

pub struct Canvas;

impl Canvas {
    pub fn as_proto() -> WieJavaClassProto {
        let mut proto = MidpCanvas::as_proto();
        proto.methods.extend([
            JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
            JavaMethodProto::new("getCommandBarHeight", "()I", Self::get_command_bar_height, MethodAccessFlags::empty()),
        ]);
        proto
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getHeight({this:?})");

        let height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let fullscreen: bool = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "isFullScreen", "()Z", ())
            .await?;
        let commands: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        if fullscreen || commands > 0 {
            return Ok(height);
        }
        let command_height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getCommandBarHeight", "()I", ())
            .await?;
        // SK-VM FAQ 4.8 keeps the command line paintable beyond the reported Canvas height.
        Ok((height - command_height).max(0))
    }

    async fn get_command_bar_height(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getCommandBarHeight({this:?})");

        Ok(16)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::{TestPlatform, run_jvm_test_with_system};
    use wie_backend::Platform;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::{
        lcdui::{Command, Display, Graphics, Image},
        midlet::MIDlet,
    };

    use crate::get_midp_protos;

    struct RecordingCanvas;

    impl RecordingCanvas {
        async fn size_changed(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _width: i32, height: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "test/CommandAreaCanvas", "lastHeight", "I", height).await
        }

        async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
            let width: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "getWidth", "()I", ())
                .await?;
            let height: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "getHeight", "()I", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                .await?;
            jvm.invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (0, 0, width, height + 16),
            )
            .await
        }
    }

    async fn assert_height(jvm: &Jvm, canvas: &ClassInstanceRef<RecordingCanvas>, height: i32) -> JvmResult<()> {
        assert_eq!(
            jvm.invoke_virtual::<_, i32>(canvas, "javax/microedition/lcdui/Canvas", "getHeight", "()I", ())
                .await?,
            height
        );
        assert_eq!(jvm.get_field::<i32>(canvas, "test/CommandAreaCanvas", "lastHeight", "I").await?, height);
        Ok(())
    }

    #[test]
    fn legacy_canvas_height_excludes_the_renderable_command_area() {
        let canvas_proto = WieJavaClassProto {
            name: "test/CommandAreaCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("sizeChanged", "(II)V", RecordingCanvas::size_changed, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    RecordingCanvas::paint,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![JavaFieldProto::new("lastHeight", "I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let midlet_proto = WieJavaClassProto {
            name: "test/CommandAreaMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let platform = TestPlatform::new();
        platform.screen().resize(240, 320).unwrap();
        run_jvm_test_with_system(
            Box::new([get_midp_protos().into(), [canvas_proto, midlet_proto].into()]),
            Box::new(platform),
            |jvm, _system| async move {
                let midlet: ClassInstanceRef<MIDlet> = jvm.instantiate_class("test/CommandAreaMidlet").await?.into();
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let display: ClassInstanceRef<Display> = MIDlet::display(&jvm, &midlet).await?;
                let canvas: ClassInstanceRef<RecordingCanvas> = jvm.instantiate_class("test/CommandAreaCanvas").await?.into();
                let _: () = jvm
                    .invoke_special(&canvas, "javax/microedition/lcdui/Canvas", "<init>", "()V", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (canvas.clone(),),
                    )
                    .await?;
                assert_height(&jvm, &canvas, 304).await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (0, 0, 240, 320))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                    .await?;
                let screen: ClassInstanceRef<Image> = jvm
                    .get_field(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "screenImage",
                        "Ljavax/microedition/lcdui/Image;",
                    )
                    .await?;
                let bottom = Image::image(&jvm, &screen).await?.get_pixel(239, 319);
                assert_eq!((bottom.r, bottom.g, bottom.b), (0x12, 0x34, 0x56));

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
                assert_height(&jvm, &canvas, 304).await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                    .await?;
                let screen = Image::image(&jvm, &screen).await?;
                let last_content = screen.get_pixel(239, 303);
                let command_pixel = screen.get_pixel(239, 304);
                assert_eq!((last_content.r, last_content.g, last_content.b), (0x12, 0x34, 0x56));
                assert_ne!((command_pixel.r, command_pixel.g, command_pixel.b), (0x12, 0x34, 0x56));
                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "javax/microedition/lcdui/Displayable",
                        "removeCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command,),
                    )
                    .await?;
                assert_height(&jvm, &canvas, 304).await?;

                for (fullscreen, height) in [(true, 320), (false, 304)] {
                    let _: () = jvm
                        .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "setFullScreenMode", "(Z)V", (fullscreen,))
                        .await?;
                    assert_height(&jvm, &canvas, height).await?;
                }
                Ok(())
            },
        )
        .unwrap();
    }
}
