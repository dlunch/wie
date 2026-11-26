use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics};
use wie_midp::classes::net::wie::MIDPKeyCode;

// class com.xce.lcdui.XTextField
#[java_class(name = "com/xce/lcdui/XTextField", public)]
pub struct XTextField {
    #[field(private)]
    text: ClassInstanceRef<String>,
    #[field(private)]
    maxSize: i32,
    #[field(private)]
    constraints: i32,
    #[field(private)]
    focus: bool,
    #[field(private)]
    x: i32,
    #[field(private)]
    y: i32,
    #[field(private)]
    width: i32,
    #[field(private)]
    height: i32,
    #[field(private)]
    canvas: ClassInstanceRef<Canvas>,
}

#[java_methods]
impl XTextField {
    #[method(constructor, public)]
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
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;", text)
            .await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "constraints", "I", constraints)
            .await?;
        jvm.put_field(
            &mut this,
            "com/xce/lcdui/XTextField",
            "canvas",
            "Ljavax/microedition/lcdui/Canvas;",
            canvas,
        )
        .await?;

        Ok(())
    }

    async fn truncate_text(jvm: &Jvm, text: ClassInstanceRef<String>, max_size: i32) -> JvmResult<ClassInstanceRef<String>> {
        let length: i32 = jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await?;
        if length > max_size {
            jvm.invoke_virtual(&text, "java/lang/String", "substring", "(II)Ljava/lang/String;", (0, max_size))
                .await
        } else {
            Ok(text)
        }
    }

    #[method(public)]
    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "com/xce/lcdui/XTextField", "maxSize", "I").await
    }

    #[method(public)]
    async fn get_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("com.xce.lcdui.XTextField::getText({this:?})");
        jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn has_focus(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "com/xce/lcdui/XTextField", "focus", "Z").await
    }

    #[method(public)]
    async fn input_char(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: JavaChar) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::inputChar({this:?}, {key})");

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await?;
        let length: i32 = jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await?;
        let max_size: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "maxSize", "I").await?;
        if length >= max_size {
            return Ok(());
        }

        let char_string: ClassInstanceRef<String> = jvm.invoke_static("java/lang/String", "valueOf", "(C)Ljava/lang/String;", (key,)).await?;
        let text: ClassInstanceRef<String> = jvm
            .invoke_virtual(
                &text,
                "java/lang/String",
                "concat",
                "(Ljava/lang/String;)Ljava/lang/String;",
                (char_string,),
            )
            .await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;", text)
            .await
    }

    #[method(public)]
    async fn set_focus(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setFocus({this:?}, {focus})");
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "focus", "Z", focus).await
    }

    #[method(public)]
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

        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "x", "I", x).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "y", "I", y).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "width", "I", width).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "height", "I", height).await
    }

    #[method(public)]
    async fn key_pressed(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyPressed({this:?}, {key_code})");

        let focus: bool = jvm.get_field(&this, "com/xce/lcdui/XTextField", "focus", "Z").await?;
        if !focus {
            return Ok(());
        }

        if key_code == MIDPKeyCode::CLEAR as i32 {
            let text: ClassInstanceRef<String> = jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await?;
            let length: i32 = jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await?;
            if length > 0 {
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&text, "java/lang/String", "substring", "(II)Ljava/lang/String;", (0, length - 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&this, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (text,))
                    .await?;
            }
            return Ok(());
        }
        if !(32..=126).contains(&key_code) {
            return Ok(());
        }
        let constraints: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "constraints", "I").await?;
        // MIDP NUMERIC, PHONENUMBER, and DECIMAL fields take the typed character directly.
        if matches!(constraints & 0xffff, 2 | 3 | 5) {
            return Self::input_char(jvm, context, this, key_code as JavaChar).await;
        }
        let text: ClassInstanceRef<String> = jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await?;
        let initial = JavaLangString::to_rust_string(jvm, &text).await?;
        if let Some(text) = context.system().platform().text_input(&initial).await {
            let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, &text).await?.into();
            let _: () = jvm
                .invoke_virtual(&this, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (text,))
                .await?;
        }
        Ok(())
    }

    #[method(public)]
    async fn key_repeated(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyRepeated({this:?}, {key_code})");

        // The text input handler ignores key repeats, so a held key types its character once.
        Ok(())
    }

    #[method(public)]
    async fn key_released(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XTextField::keyReleased({this:?}, {key_code})");

        Ok(())
    }

    #[method(public)]
    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::paint({this:?}, {graphics:?})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await?;
        let x: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "height", "I").await?;
        let clip_x: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
            .await?;
        let clip_y: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
            .await?;
        let clip_width: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
            .await?;
        let clip_height: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
            .await?;

        // The field draws its own box: light gray with a black border while focused, white with a
        // light gray border otherwise, and black text inside it.
        let focus: bool = jvm.get_field(&this, "com/xce/lcdui/XTextField", "focus", "Z").await?;
        let (background, border) = if focus { (0xaaaaaa, 0x000000) } else { (0xffffff, 0xaaaaaa) };
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (background,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (x, y, width + 1, height + 1),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (border,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x000000,))
            .await?;

        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "clipRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await?;
        let draw_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawString",
                "(Ljava/lang/String;III)V",
                (text, x + 1, y + 1, 20),
            )
            .await;
        let restore_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setClip",
                "(IIII)V",
                (clip_x, clip_y, clip_width, clip_height),
            )
            .await;
        draw_result?;
        restore_result
    }

    #[method(public)]
    async fn repaint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::repaint({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm
            .get_field(&this, "com/xce/lcdui/XTextField", "canvas", "Ljavax/microedition/lcdui/Canvas;")
            .await?;
        let x: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "height", "I").await?;
        jvm.invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (x, y, width, height))
            .await
    }

    #[method(public)]
    async fn set_max_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setMaxSize({this:?}, {max_size})");

        if max_size < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must not be negative").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;", text)
            .await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "maxSize", "I", max_size).await
    }

    #[method(public)]
    async fn set_text(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setText({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "text is null").await);
        }

        let max_size: i32 = jvm.get_field(&this, "com/xce/lcdui/XTextField", "maxSize", "I").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "com/xce/lcdui/XTextField", "text", "Ljava/lang/String;", text)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaChar, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{java_class, java_methods};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_jvm_support::WieJvmContext;
    use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics, Image};

    use crate::classes::{
        com::xce::lcdui::{TextComponent, TextComponentHandler},
        net::wie::WieTextComponentHandler,
    };

    use super::XTextField;

    #[java_class(name = "test/TrackingCanvas", extends = "javax/microedition/lcdui/Canvas", public)]
    struct TrackingCanvas {
        #[field(public)]
        repaintCount: i32,
        #[field(public)]
        repaintX: i32,
        #[field(public)]
        repaintY: i32,
        #[field(public)]
        repaintWidth: i32,
        #[field(public)]
        repaintHeight: i32,
    }
    #[java_class(name = "javax/microedition/lcdui/TrackingGraphics", extends = "javax/microedition/lcdui/Graphics", public)]
    struct TrackingGraphics {
        #[field(public)]
        observedClipX: i32,
        #[field(public)]
        observedClipY: i32,
        #[field(public)]
        observedClipWidth: i32,
        #[field(public)]
        observedClipHeight: i32,
    }
    #[java_class(
        name = "test/TrackingComponent",
        extends = "com/xce/lcdui/XTextField",
        implements("com/xce/lcdui/TextComponent"),
        public
    )]
    struct TrackingComponent;

    #[java_methods]
    impl TrackingComponent {
        #[method(public)]
        async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            let text: ClassInstanceRef<String> = jvm
                .invoke_virtual(&this, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                .await?;
            jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await
        }

        #[method(name = "getConstraints", public)]
        async fn constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            jvm.get_field(&this, "com/xce/lcdui/XTextField", "constraints", "I").await
        }

        #[method(name = "getMaxSize", public)]
        async fn max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            jvm.invoke_special(&this, "com/xce/lcdui/XTextField", "getMaxSize", "()I", ()).await
        }

        #[method(public)]
        async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "com/xce/lcdui/XTextField", "repaint", "()V", ()).await
        }

        #[method(public)]
        async fn insert(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, value: JavaChar) -> JvmResult<()> {
            jvm.invoke_virtual(&this, "com/xce/lcdui/XTextField", "inputChar", "(C)V", (value,)).await
        }

        #[method(public)]
        async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let text: ClassInstanceRef<String> = jvm
                .invoke_virtual(&this, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                .await?;
            let size: i32 = jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await?;
            let replacement: ClassInstanceRef<String> = jvm
                .invoke_virtual(&text, "java/lang/String", "substring", "(II)Ljava/lang/String;", (0, size - 1))
                .await?;
            jvm.invoke_virtual(&this, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (replacement,))
                .await
        }
    }

    #[java_methods]
    impl TrackingCanvas {
        #[method(constructor, public)]
        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        #[method(public)]
        async fn repaint(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "test/TrackingCanvas", "repaintCount", "I").await?;
            jvm.put_field(&mut this, "test/TrackingCanvas", "repaintCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "test/TrackingCanvas", "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "test/TrackingCanvas", "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "test/TrackingCanvas", "repaintWidth", "I", width).await?;
            jvm.put_field(&mut this, "test/TrackingCanvas", "repaintHeight", "I", height).await
        }

        #[method(public)]
        async fn paint(
            _jvm: &Jvm,
            _context: &mut WieJvmContext,
            _this: ClassInstanceRef<Self>,
            _graphics: ClassInstanceRef<wie_midp::classes::javax::microedition::lcdui::Graphics>,
        ) -> JvmResult<()> {
            Ok(())
        }
    }

    #[java_methods]
    impl TrackingGraphics {
        #[method(constructor, public)]
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

        #[method(public)]
        async fn draw_string(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            _string: ClassInstanceRef<String>,
            _x: i32,
            _y: i32,
            _anchor: i32,
        ) -> JvmResult<()> {
            let clip_x: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                .await?;
            let clip_y: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                .await?;
            let clip_width: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                .await?;
            let clip_height: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TrackingGraphics", "observedClipX", "I", clip_x)
                .await?;
            jvm.put_field(&mut this, "javax/microedition/lcdui/TrackingGraphics", "observedClipY", "I", clip_y)
                .await?;
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/TrackingGraphics",
                "observedClipWidth",
                "I",
                clip_width,
            )
            .await?;
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/TrackingGraphics",
                "observedClipHeight",
                "I",
                clip_height,
            )
            .await
        }
    }

    #[test]
    fn text_state_input_limit_and_repaint_work_as_one_flow() {
        let result = run_jvm_test_with_system(
            Box::new([
                wie_midp::get_protos().into(),
                [
                    XTextField::as_proto(),
                    TrackingCanvas::as_proto(),
                    TrackingGraphics::as_proto(),
                    TrackingComponent::as_proto(),
                    TextComponent::as_proto(),
                    TextComponentHandler::as_proto(),
                    WieTextComponentHandler::as_proto(),
                ]
                .into(),
            ]),
            Box::new(TestPlatform::new().with_text_input("가나다")),
            |jvm, _| async move {
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let initial: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "abc").await?.into();
                let field: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (initial, 5, 2, canvas.clone()),
                    )
                    .await?
                    .into();

                let max_size: i32 = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "getMaxSize", "()I", ()).await?;
                let focused: bool = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "hasFocus", "()Z", ()).await?;
                assert_eq!(max_size, 5);
                assert!(!focused);

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", ('z' as i32,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abc");

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setFocus", "(Z)V", (true,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (3, 5, 40, 12))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", ('d' as i32,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyRepeated", "(I)V", ('e' as i32,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "inputChar", "(C)V", ('f' as JavaChar,))
                    .await?;

                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abcdf");
                let _: () = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", (8,)).await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abcd");
                assert!(
                    jvm.invoke_virtual::<_, bool>(&field, "com/xce/lcdui/XTextField", "hasFocus", "()Z", ())
                        .await?
                );

                for bounds in [(9, 8, -1, 4), (7, 6, 4, -1)] {
                    let bounds_result: JvmResult<()> = jvm
                        .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", bounds)
                        .await;
                    let Err(JavaError::JavaException(exception)) = bounds_result else {
                        panic!("XTextField.setBounds accepted a negative size");
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                }

                let _: () = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "setMaxSize", "(I)V", (3,)).await?;
                let replacement: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "wxyz").await?.into();
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (replacement,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "wxy");

                let _: () = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "repaint", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintCount", "I").await?, 1);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintX", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintY", "I").await?, 5);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintWidth", "I").await?, 40);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintHeight", "I").await?, 12);

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (1, 2, 0, 0))
                    .await?;

                let negative_result: JvmResult<()> = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "setMaxSize", "(I)V", (-1,)).await;
                let Err(JavaError::JavaException(exception)) = negative_result else {
                    panic!("XTextField.setMaxSize accepted a negative value");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let null_text = ClassInstanceRef::<String>::new(None);
                let null_result: JvmResult<()> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (null_text,))
                    .await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("XTextField.setText accepted null");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                let mut component = jvm.instantiate_class("test/TrackingComponent").await?;
                let empty = JavaLangString::from_rust_string(&jvm, "").await?;
                let _: () = jvm
                    .invoke_special(
                        &component,
                        "com/xce/lcdui/XTextField",
                        "<init>",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (empty, 2, 2, canvas.clone()),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&component, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (0, 0, 20, 10))
                    .await?;
                let handler: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(
                        "com/xce/lcdui/TextComponentHandler",
                        "getTextComponentHandler",
                        "()Lcom/xce/lcdui/TextComponentHandler;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &handler,
                        "com/xce/lcdui/TextComponentHandler",
                        "setTextComponent",
                        "(Lcom/xce/lcdui/TextComponent;)V",
                        (component.clone(),),
                    )
                    .await?;
                for (key, expected) in [(49, "1"), (50, "12"), (51, "12"), (8, "1")] {
                    assert!(
                        jvm.invoke_virtual::<_, bool>(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", (key,))
                            .await?
                    );
                    let text: ClassInstanceRef<String> = jvm
                        .invoke_virtual(&component, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                        .await?;
                    assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, expected);
                }
                jvm.put_field(&mut component, "com/xce/lcdui/XTextField", "constraints", "I", 0x20000)
                    .await?;
                let _: bool = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", (57,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&component, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "1");
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintCount", "I").await?, 4);

                // Any key on a free-text field asks the host for text and inserts what fits.
                jvm.put_field(&mut component, "com/xce/lcdui/XTextField", "constraints", "I", 0).await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&handler, "com/xce/lcdui/TextComponentHandler", "keyPressed", "(I)Z", (-5,))
                        .await?
                );
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&component, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "1가");
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TrackingCanvas", "repaintCount", "I").await?, 5);

                let existing: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "x").await?.into();
                let free_text = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (existing, 3, 0, canvas),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&free_text, "com/xce/lcdui/XTextField", "setFocus", "(Z)V", (true,))
                    .await?;
                // A typing key edits the current text in the host input; the result replaces it up to the limit.
                let _: () = jvm
                    .invoke_virtual(&free_text, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", (57,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&free_text, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "x가나");

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

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (3, 5, 4, 6))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (1, 2, 10, 11))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &field,
                        "com/xce/lcdui/XTextField",
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        (graphics.clone(),),
                    )
                    .await?;

                assert_eq!(
                    jvm.get_field::<i32>(&graphics, "javax/microedition/lcdui/TrackingGraphics", "observedClipX", "I")
                        .await?,
                    3
                );
                assert_eq!(
                    jvm.get_field::<i32>(&graphics, "javax/microedition/lcdui/TrackingGraphics", "observedClipY", "I")
                        .await?,
                    5
                );
                assert_eq!(
                    jvm.get_field::<i32>(&graphics, "javax/microedition/lcdui/TrackingGraphics", "observedClipWidth", "I")
                        .await?,
                    4
                );
                assert_eq!(
                    jvm.get_field::<i32>(&graphics, "javax/microedition/lcdui/TrackingGraphics", "observedClipHeight", "I")
                        .await?,
                    6
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                        .await?,
                    1
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                        .await?,
                    2
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                        .await?,
                    10
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                        .await?,
                    11
                );

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
