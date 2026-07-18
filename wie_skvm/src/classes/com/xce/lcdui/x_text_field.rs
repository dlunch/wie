use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics};

// class com.xce.lcdui.XTextField
pub struct XTextField;

impl XTextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XTextField",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getMaxSize", "()I", Self::get_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getText", "()Ljava/lang/String;", Self::get_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("inputChar", "(C)V", Self::input_char, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBounds", "(IIII)V", Self::set_bounds, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "(Z)V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxSize", "(I)V", Self::set_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setText", "(Ljava/lang/String;)V", Self::set_text, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("focus", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("x", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("canvas", "Ljavax/microedition/lcdui/Canvas;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
        canvas: ClassInstanceRef<Canvas>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::<init>({this:?}, {text:?}, {max_size}, {constraints}, {canvas:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if text.is_null() || canvas.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "text and canvas must not be null").await);
        }
        if max_size < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must not be negative").await);
        }

        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        jvm.put_field(&mut this, "canvas", "Ljavax/microedition/lcdui/Canvas;", canvas).await?;

        Ok(())
    }

    async fn truncate_text(jvm: &Jvm, text: ClassInstanceRef<String>, max_size: i32) -> JvmResult<ClassInstanceRef<String>> {
        let length: i32 = jvm.invoke_virtual(&text, "length", "()I", ()).await?;
        if length > max_size {
            jvm.invoke_virtual(&text, "substring", "(II)Ljava/lang/String;", (0, max_size)).await
        } else {
            Ok(text)
        }
    }

    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxSize", "I").await
    }

    async fn get_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("com.xce.lcdui.XTextField::getText({this:?})");
        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn has_focus(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "focus", "Z").await
    }

    async fn input_char(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: JavaChar) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::inputChar({this:?}, {key})");

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let length: i32 = jvm.invoke_virtual(&text, "length", "()I", ()).await?;
        let max_size: i32 = jvm.get_field(&this, "maxSize", "I").await?;
        if length >= max_size {
            return Ok(());
        }

        let char_string: ClassInstanceRef<String> = jvm.invoke_static("java/lang/String", "valueOf", "(C)Ljava/lang/String;", (key,)).await?;
        let text: ClassInstanceRef<String> = jvm
            .invoke_virtual(&text, "concat", "(Ljava/lang/String;)Ljava/lang/String;", (char_string,))
            .await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await
    }

    async fn set_focus(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setFocus({this:?}, {focus})");
        jvm.put_field(&mut this, "focus", "Z", focus).await
    }

    async fn set_bounds(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setBounds({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        jvm.put_field(&mut this, "x", "I", x).await?;
        jvm.put_field(&mut this, "y", "I", y).await?;
        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await
    }

    async fn key_pressed(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyPressed({this:?}, {key_code})");

        let focus: bool = jvm.get_field(&this, "focus", "Z").await?;
        if !focus {
            return Ok(());
        }

        if (32..=126).contains(&key_code) {
            return Self::input_char(jvm, context, this, key_code as JavaChar).await;
        }
        Ok(())
    }

    async fn key_repeated(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyRepeated({this:?}, {key_code})");
        Self::key_pressed(jvm, context, this, key_code).await
    }

    async fn key_released(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::keyReleased({this:?}, {key_code})");

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::paint({this:?}, {graphics:?})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let x: i32 = jvm.get_field(&this, "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
        let clip_x: i32 = jvm.invoke_virtual(&graphics, "getClipX", "()I", ()).await?;
        let clip_y: i32 = jvm.invoke_virtual(&graphics, "getClipY", "()I", ()).await?;
        let clip_width: i32 = jvm.invoke_virtual(&graphics, "getClipWidth", "()I", ()).await?;
        let clip_height: i32 = jvm.invoke_virtual(&graphics, "getClipHeight", "()I", ()).await?;

        let _: () = jvm.invoke_virtual(&graphics, "clipRect", "(IIII)V", (x, y, width, height)).await?;
        let draw_result: JvmResult<()> = jvm
            .invoke_virtual(&graphics, "drawString", "(Ljava/lang/String;III)V", (text, x, y, 20))
            .await;
        let restore_result: JvmResult<()> = jvm
            .invoke_virtual(&graphics, "setClip", "(IIII)V", (clip_x, clip_y, clip_width, clip_height))
            .await;
        draw_result?;
        restore_result
    }

    async fn repaint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::repaint({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm.get_field(&this, "canvas", "Ljavax/microedition/lcdui/Canvas;").await?;
        let x: i32 = jvm.get_field(&this, "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
        jvm.invoke_virtual(&canvas, "repaint", "(IIII)V", (x, y, width, height)).await
    }

    async fn set_max_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setMaxSize({this:?}, {max_size})");

        if max_size < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must not be negative").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await
    }

    async fn set_text(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setText({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "text is null").await);
        }

        let max_size: i32 = jvm.get_field(&this, "maxSize", "I").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use java_class_proto::{JavaFieldProto, JavaMethodProto};
    use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, JavaChar, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics, Image};

    use super::XTextField;

    struct TrackingCanvas;
    struct TrackingGraphics;

    impl TrackingCanvas {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TrackingCanvas",
                parent_class: Some("javax/microedition/lcdui/Canvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("repaintCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintHeight", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        async fn repaint(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "repaintCount", "I").await?;
            jvm.put_field(&mut this, "repaintCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "repaintWidth", "I", width).await?;
            jvm.put_field(&mut this, "repaintHeight", "I", height).await
        }

        async fn paint(
            _jvm: &Jvm,
            _context: &mut WieJvmContext,
            _this: ClassInstanceRef<Self>,
            _graphics: ClassInstanceRef<wie_midp::classes::javax::microedition::lcdui::Graphics>,
        ) -> JvmResult<()> {
            Ok(())
        }
    }

    impl TrackingGraphics {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "javax/microedition/lcdui/TrackingGraphics",
                parent_class: Some("javax/microedition/lcdui/Graphics"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("observedClipX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipHeight", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (20, 20),
                )
                .await?;
            jvm.invoke_special(
                &this,
                "javax/microedition/lcdui/Graphics",
                "<init>",
                "(Ljavax/microedition/lcdui/Image;)V",
                (image,),
            )
            .await
        }

        async fn draw_string(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            _string: ClassInstanceRef<String>,
            _x: i32,
            _y: i32,
            _anchor: i32,
        ) -> JvmResult<()> {
            let clip_x: i32 = jvm.invoke_virtual(&this, "getClipX", "()I", ()).await?;
            let clip_y: i32 = jvm.invoke_virtual(&this, "getClipY", "()I", ()).await?;
            let clip_width: i32 = jvm.invoke_virtual(&this, "getClipWidth", "()I", ()).await?;
            let clip_height: i32 = jvm.invoke_virtual(&this, "getClipHeight", "()I", ()).await?;
            jvm.put_field(&mut this, "observedClipX", "I", clip_x).await?;
            jvm.put_field(&mut this, "observedClipY", "I", clip_y).await?;
            jvm.put_field(&mut this, "observedClipWidth", "I", clip_width).await?;
            jvm.put_field(&mut this, "observedClipHeight", "I", clip_height).await
        }
    }

    #[test]
    fn text_state_input_limit_and_repaint_work_as_one_flow() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [XTextField::as_proto(), TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let initial: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "abc").await?.into();
                let field: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (initial, 5, 0, canvas.clone()),
                    )
                    .await?
                    .into();

                let max_size: i32 = jvm.invoke_virtual(&field, "getMaxSize", "()I", ()).await?;
                let focused: bool = jvm.invoke_virtual(&field, "hasFocus", "()Z", ()).await?;
                assert_eq!(max_size, 5);
                assert!(!focused);

                let _: () = jvm.invoke_virtual(&field, "keyPressed", "(I)V", ('z' as i32,)).await?;
                let text: ClassInstanceRef<String> = jvm.invoke_virtual(&field, "getText", "()Ljava/lang/String;", ()).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abc");

                let _: () = jvm.invoke_virtual(&field, "setFocus", "(Z)V", (true,)).await?;
                let _: () = jvm.invoke_virtual(&field, "setBounds", "(IIII)V", (3, 5, 40, 12)).await?;
                let _: () = jvm.invoke_virtual(&field, "keyPressed", "(I)V", ('d' as i32,)).await?;
                let _: () = jvm.invoke_virtual(&field, "keyRepeated", "(I)V", ('e' as i32,)).await?;
                let _: () = jvm.invoke_virtual(&field, "inputChar", "(C)V", ('f' as JavaChar,)).await?;

                let text: ClassInstanceRef<String> = jvm.invoke_virtual(&field, "getText", "()Ljava/lang/String;", ()).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abcde");
                assert!(jvm.invoke_virtual::<_, bool>(&field, "hasFocus", "()Z", ()).await?);

                for bounds in [(9, 8, -1, 4), (7, 6, 4, -1)] {
                    let bounds_result: JvmResult<()> = jvm.invoke_virtual(&field, "setBounds", "(IIII)V", bounds).await;
                    let Err(JavaError::JavaException(exception)) = bounds_result else {
                        panic!("XTextField.setBounds accepted a negative size");
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                }

                let _: () = jvm.invoke_virtual(&field, "setMaxSize", "(I)V", (3,)).await?;
                let replacement: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "wxyz").await?.into();
                let _: () = jvm.invoke_virtual(&field, "setText", "(Ljava/lang/String;)V", (replacement,)).await?;
                let text: ClassInstanceRef<String> = jvm.invoke_virtual(&field, "getText", "()Ljava/lang/String;", ()).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "wxy");

                let _: () = jvm.invoke_virtual(&field, "repaint", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 1);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintX", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintY", "I").await?, 5);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintWidth", "I").await?, 40);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintHeight", "I").await?, 12);

                let _: () = jvm.invoke_virtual(&field, "setBounds", "(IIII)V", (1, 2, 0, 0)).await?;

                let negative_result: JvmResult<()> = jvm.invoke_virtual(&field, "setMaxSize", "(I)V", (-1,)).await;
                let Err(JavaError::JavaException(exception)) = negative_result else {
                    panic!("XTextField.setMaxSize accepted a negative value");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let null_text = ClassInstanceRef::<String>::new(None);
                let null_result: JvmResult<()> = jvm.invoke_virtual(&field, "setText", "(Ljava/lang/String;)V", (null_text,)).await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("XTextField.setText accepted null");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn paint_uses_the_bounds_clip_and_restores_the_original_clip() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [XTextField::as_proto(), TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "long text").await?.into();
                let field: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (text, 20, 0, canvas),
                    )
                    .await?
                    .into();
                let graphics: ClassInstanceRef<Graphics> = jvm.new_class("javax/microedition/lcdui/TrackingGraphics", "()V", ()).await?.into();

                let _: () = jvm.invoke_virtual(&field, "setBounds", "(IIII)V", (3, 5, 4, 6)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (1, 2, 10, 11)).await?;
                let _: () = jvm
                    .invoke_virtual(&field, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics.clone(),))
                    .await?;

                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipX", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipY", "I").await?, 5);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipWidth", "I").await?, 4);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipHeight", "I").await?, 6);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipX", "()I", ()).await?, 1);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipY", "()I", ()).await?, 2);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipWidth", "()I", ()).await?, 10);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipHeight", "()I", ()).await?, 11);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
