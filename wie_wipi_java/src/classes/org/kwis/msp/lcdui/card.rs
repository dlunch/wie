use alloc::{format, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Canvas;

use crate::classes::org::kwis::msp::lcdui::Display;

// abstract class org.kwis.msp.lcdui.Card
pub struct Card;

impl Card {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Card",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["org/kwis/msp/lcdui/JletEventListener"],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(I)V", Self::init_int, Default::default()),
                JavaMethodProto::new("<init>", "(Z)V", Self::init_transparent, Default::default()),
                JavaMethodProto::new("<init>", "(IIII)V", Self::init_with_bounds, Default::default()),
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;)V", Self::init_with_display, Default::default()),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Display;IIII)V",
                    Self::init_with_display_and_bounds,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                    Self::init_with_display_bounds_and_transparency,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("move", "(II)V", Self::move_card, Default::default()),
                JavaMethodProto::new("resize", "(II)V", Self::resize, Default::default()),
                JavaMethodProto::new("getX", "()I", Self::get_x, Default::default()),
                JavaMethodProto::new("getY", "()I", Self::get_y, Default::default()),
                JavaMethodProto::new("pointerNotify", "(III)Z", Self::pointer_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("getDisplay", "()Lorg/kwis/msp/lcdui/Display;", Self::get_display, Default::default()),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, Default::default()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, Default::default()),
                JavaMethodProto::new("repaint", "()V", Self::repaint, Default::default()),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, Default::default()),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, Default::default()),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, Default::default()),
                JavaMethodProto::new_abstract("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Default::default()),
                // wie private
                JavaMethodProto::new("setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", Self::set_canvas, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("canvas", "Ljavax/microedition/lcdui/Canvas;", Default::default()),
                JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", Default::default()),
                JavaFieldProto::new("x", "I", Default::default()),
                JavaFieldProto::new("y", "I", Default::default()),
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
                JavaFieldProto::new("transparent", "Z", Default::default()),
            ],
            access_flags: ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.Card::<init>({this:?})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
            .await?;

        Ok(())
    }

    // not in reference, but called by some clet
    async fn init_int(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, a0: i32) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.Card::<init>({this:?}, {a0})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
            .await?;

        Ok(())
    }

    async fn init_transparent(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, transparent: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::<init>({this:?}, {transparent})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        if display.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "display is null").await);
        }

        let width: i32 = jvm.invoke_virtual(&display, "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "getHeight", "()I", []).await?;
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, 0, 0, width, height, transparent),
            )
            .await?;

        Ok(())
    }

    async fn init_with_bounds(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Card>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::<init>({this:?}, {x}, {y}, {width}, {height})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, x, y, width, height, false),
            )
            .await?;

        Ok(())
    }

    async fn init_with_display(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, display: ClassInstanceRef<Display>) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Card::<init>({this:?}, {display:?})");
        tracing::debug!("{log}");

        if display.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "display is null").await);
        }

        let width: i32 = jvm.invoke_virtual(&display, "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "getHeight", "()I", []).await?;
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, 0, 0, width, height, false),
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_display_and_bounds(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Card>,
        display: ClassInstanceRef<Display>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Card::<init>({this:?}, {display:?}, {x}, {y}, {width}, {height})");
        tracing::debug!("{log}");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, x, y, width, height, false),
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_display_bounds_and_transparency(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Card>,
        display: ClassInstanceRef<Display>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        transparent: bool,
    ) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Card::<init>({this:?}, {display:?}, {x}, {y}, {width}, {height}, {transparent})");
        tracing::debug!("{log}");

        if display.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "display is null").await);
        }
        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "display", "Lorg/kwis/msp/lcdui/Display;", display).await?;
        jvm.put_field(&mut this, "x", "I", x).await?;
        jvm.put_field(&mut this, "y", "I", y).await?;
        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;
        jvm.put_field(&mut this, "transparent", "Z", transparent).await?;

        Ok(())
    }

    async fn move_card(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::move({this:?}, {x}, {y})");

        jvm.put_field(&mut this, "x", "I", x).await?;
        jvm.put_field(&mut this, "y", "I", y).await
    }

    async fn resize(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::resize({this:?}, {width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await
    }

    async fn get_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getX({this:?})");

        jvm.get_field(&this, "x", "I").await
    }

    async fn get_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getY({this:?})");

        jvm.get_field(&this, "y", "I").await
    }

    async fn pointer_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, r#type: i32, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::pointerNotify({this:?}, {type}, {x}, {y})");

        Ok(false)
    }

    async fn get_display(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<ClassInstanceRef<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getDisplay({this:?})");

        jvm.get_field(&this, "display", "Lorg/kwis/msp/lcdui/Display;").await
    }

    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Card::isShown({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm.get_field(&this, "canvas", "Ljavax/microedition/lcdui/Canvas;").await?;
        Ok(!canvas.is_null())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({this:?})");

        jvm.get_field(&this, "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({this:?})");

        jvm.get_field(&this, "h", "I").await
    }

    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({this:?})");

        let width: i32 = jvm.get_field(&this, "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "h", "I").await?;

        let _: () = jvm.invoke_virtual(&this, "repaint", "(IIII)V", (0, 0, width, height)).await?;

        Ok(())
    }

    async fn repaint_with_area(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Card>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({this:?}, {x}, {y}, {width}, {height})");

        let canvas: ClassInstanceRef<Canvas> = jvm.get_field(&this, "canvas", "Ljavax/microedition/lcdui/Canvas;").await?;
        if canvas.is_null() || width <= 0 || height <= 0 {
            return Ok(());
        }

        let card_x: i32 = jvm.get_field(&this, "x", "I").await?;
        let card_y: i32 = jvm.get_field(&this, "y", "I").await?;
        let card_width: i32 = jvm.get_field(&this, "w", "I").await?;
        let card_height: i32 = jvm.get_field(&this, "h", "I").await?;

        let repaint_x = i64::from(x).max(0).min(i64::from(card_width));
        let repaint_y = i64::from(y).max(0).min(i64::from(card_height));
        let repaint_right = (i64::from(x) + i64::from(width)).max(0).min(i64::from(card_width));
        let repaint_bottom = (i64::from(y) + i64::from(height)).max(0).min(i64::from(card_height));
        if repaint_right <= repaint_x || repaint_bottom <= repaint_y {
            return Ok(());
        }

        let _: () = jvm
            .invoke_virtual(
                &canvas,
                "repaint",
                "(IIII)V",
                (
                    card_x.saturating_add(repaint_x as i32),
                    card_y.saturating_add(repaint_y as i32),
                    (repaint_right - repaint_x) as i32,
                    (repaint_bottom - repaint_y) as i32,
                ),
            )
            .await?;

        Ok(())
    }

    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::serviceRepaints({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm.get_field(&this, "canvas", "Ljavax/microedition/lcdui/Canvas;").await?;
        if !canvas.is_null() {
            let _: () = jvm.invoke_virtual(&canvas, "serviceRepaints", "()V", ()).await?;
        }

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, show: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::showNotify({this:?}, {show})");

        Ok(())
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Card::keyNotify({this:?}, {type}, {key})");

        Ok(false)
    }

    async fn set_canvas(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, canvas: ClassInstanceRef<Canvas>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::setCanvas({this:?}, {canvas:?})");

        jvm.put_field(&mut this, "canvas", "Ljavax/microedition/lcdui/Canvas;", canvas).await
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};

    use java_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::{Display as MidpDisplay, Graphics as MidpGraphics, Image as MidpImage};
    use wie_util::Result;

    use crate::{
        classes::net::wie::CardCanvas,
        classes::org::kwis::msp::lcdui::{Display, Graphics},
        get_protos,
    };

    struct TestCard;

    impl TestCard {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestCard",
                parent_class: Some("org/kwis/msp/lcdui/Card"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", Self::init, Default::default()),
                    JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;)V", Self::init_with_display, Default::default()),
                    JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, Default::default()),
                    JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, Default::default()),
                    JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, Default::default()),
                    JavaMethodProto::new("notifyEvent", "(III)V", Self::notify_event, Default::default()),
                ],
                fields: vec![
                    JavaFieldProto::new("showCount", "I", Default::default()),
                    JavaFieldProto::new("hideCount", "I", Default::default()),
                    JavaFieldProto::new("keyCount", "I", Default::default()),
                    JavaFieldProto::new("notifyCount", "I", Default::default()),
                ],
                access_flags: Default::default(),
            }
        }

        #[allow(clippy::too_many_arguments)]
        async fn init(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            this: ClassInstanceRef<Self>,
            display: ClassInstanceRef<Display>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            transparent: bool,
        ) -> JvmResult<()> {
            jvm.invoke_special(
                &this,
                "org/kwis/msp/lcdui/Card",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                (display, x, y, width, height, transparent),
            )
            .await
        }

        async fn init_with_display(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            this: ClassInstanceRef<Self>,
            display: ClassInstanceRef<Display>,
        ) -> JvmResult<()> {
            jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
                .await
        }

        async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
            let x: i32 = jvm.get_field(&this, "x", "I").await?;
            let y: i32 = jvm.get_field(&this, "y", "I").await?;
            let width: i32 = jvm.get_field(&this, "w", "I").await?;
            let height: i32 = jvm.get_field(&this, "h", "I").await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getTranslateX", "()I", ()).await?, x);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getTranslateY", "()I", ()).await?, y);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getColor", "()I", ()).await?, 0);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getAlpha", "()I", ()).await?, 255);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getStrokeStyle", "()I", ()).await?, 0);
            assert!(!jvm.invoke_virtual::<_, bool>(&graphics, "isXORMode", "()Z", ()).await?);

            let transparent: bool = jvm.get_field(&this, "transparent", "Z").await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "setColor", "(I)V", (if transparent { 0xff0000 } else { 0x00ff00 },))
                .await?;
            let _: () = jvm.invoke_virtual(&graphics, "fillRect", "(IIII)V", (0, 0, width, height)).await?;

            if transparent {
                let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (7, 8)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, 1, 1)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setAlpha", "(I)V", (0,)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setStrokeStyle", "(I)V", (1,)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setXORMode", "(Z)V", (true,)).await?;
            }

            Ok(())
        }

        async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
            let _: () = jvm
                .invoke_special(&this, "org/kwis/msp/lcdui/Card", "showNotify", "(Z)V", (show,))
                .await?;
            let field = if show { "showCount" } else { "hideCount" };
            let count: i32 = jvm.get_field(&this, field, "I").await?;
            jvm.put_field(&mut this, field, "I", count + 1).await
        }

        async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: i32, _: i32) -> JvmResult<bool> {
            let count: i32 = jvm.get_field(&this, "keyCount", "I").await?;
            jvm.put_field(&mut this, "keyCount", "I", count + 1).await?;
            Ok(false)
        }

        async fn notify_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: i32, _: i32, _: i32) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "notifyCount", "I").await?;
            jvm.put_field(&mut this, "notifyCount", "I", count + 1).await
        }
    }

    struct TestCanvas;

    impl TestCanvas {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestCanvas",
                parent_class: Some("javax/microedition/lcdui/Canvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                    JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, Default::default()),
                    JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, Default::default()),
                ],
                fields: vec![
                    JavaFieldProto::new("repaintX", "I", Default::default()),
                    JavaFieldProto::new("repaintY", "I", Default::default()),
                    JavaFieldProto::new("repaintWidth", "I", Default::default()),
                    JavaFieldProto::new("repaintHeight", "I", Default::default()),
                    JavaFieldProto::new("repaintCount", "I", Default::default()),
                ],
                access_flags: Default::default(),
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        async fn paint(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, _: ClassInstanceRef<MidpGraphics>) -> JvmResult<()> {
            Ok(())
        }

        async fn repaint(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "repaintCount", "I").await?;
            jvm.put_field(&mut this, "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "repaintWidth", "I", width).await?;
            jvm.put_field(&mut this, "repaintHeight", "I", height).await?;
            jvm.put_field(&mut this, "repaintCount", "I", count + 1).await
        }
    }

    #[test]
    fn test_card_state_and_validation() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestCard::as_proto(), TestCanvas::as_proto()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let display: ClassInstanceRef<Display> = jvm.instantiate_class("org/kwis/msp/lcdui/Display").await?.into();
                let card: ClassInstanceRef<TestCard> = jvm
                    .new_class(
                        "test/TestCard",
                        "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                        (display.clone(), 3, 4, 20, 30, true),
                    )
                    .await?
                    .into();

                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getX", "()I", ()).await?, 3);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getY", "()I", ()).await?, 4);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getWidth", "()I", ()).await?, 20);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getHeight", "()I", ()).await?, 30);

                let returned_display: ClassInstanceRef<Display> =
                    jvm.invoke_virtual(&card, "getDisplay", "()Lorg/kwis/msp/lcdui/Display;", ()).await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&returned_display, "equals", "(Ljava/lang/Object;)Z", (display.clone(),))
                        .await?
                );

                let _: () = jvm.invoke_virtual(&card, "move", "(II)V", (-5, 7)).await?;
                let _: () = jvm.invoke_virtual(&card, "resize", "(II)V", (40, 50)).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getX", "()I", ()).await?, -5);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getY", "()I", ()).await?, 7);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getWidth", "()I", ()).await?, 40);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getHeight", "()I", ()).await?, 50);

                assert!(jvm.invoke_virtual::<_, ()>(&card, "resize", "(II)V", (0, 10)).await.is_err());
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getWidth", "()I", ()).await?, 40);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&card, "getHeight", "()I", ()).await?, 50);

                let invalid_size: JvmResult<ClassInstanceRef<TestCard>> = jvm
                    .new_class(
                        "test/TestCard",
                        "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                        (display.clone(), 0, 0, -1, 10, false),
                    )
                    .await
                    .map(Into::into);
                assert!(invalid_size.is_err());

                let null_display: JvmResult<ClassInstanceRef<TestCard>> = jvm
                    .new_class("test/TestCard", "(Lorg/kwis/msp/lcdui/Display;)V", [None.into()])
                    .await
                    .map(Into::into);
                assert!(null_display.is_err());

                Ok(())
            },
        )
    }

    #[test]
    fn test_card_stack_lifecycle_and_order() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestCard::as_proto(), TestCanvas::as_proto()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let midp_display: ClassInstanceRef<MidpDisplay> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &midp_display,
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (canvas.clone(),),
                    )
                    .await?;
                let display: ClassInstanceRef<Display> = jvm.instantiate_class("org/kwis/msp/lcdui/Display").await?.into();
                let first: ClassInstanceRef<TestCard> = jvm
                    .new_class(
                        "test/TestCard",
                        "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                        (display.clone(), 0, 0, 10, 10, false),
                    )
                    .await?
                    .into();
                let second: ClassInstanceRef<TestCard> = jvm
                    .new_class("test/TestCard", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", (display, 0, 0, 10, 10, false))
                    .await?
                    .into();

                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", [None.into()])
                    .await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&canvas, "countCard", "()I", ()).await?, 0);
                assert!(!jvm.invoke_virtual::<_, bool>(&first, "isShown", "()Z", ()).await?);

                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (first.clone(),))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (first.clone(),))
                    .await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&canvas, "countCard", "()I", ()).await?, 1);
                assert_eq!(jvm.get_field::<i32>(&first, "showCount", "I").await?, 1);
                assert!(jvm.invoke_virtual::<_, bool>(&first, "isShown", "()Z", ()).await?);

                let other_canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(&other_canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (first.clone(),))
                    .await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&other_canvas, "countCard", "()I", ()).await?, 0);

                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (second.clone(),))
                    .await?;
                let _: () = jvm.invoke_virtual(&canvas, "keyPressed", "(I)V", (42,)).await?;
                let _: () = jvm.invoke_virtual(&canvas, "handleNotifyEvent", "(III)V", (1, 2, 3)).await?;
                assert_eq!(jvm.get_field::<i32>(&first, "keyCount", "I").await?, 0);
                assert_eq!(jvm.get_field::<i32>(&first, "notifyCount", "I").await?, 0);
                assert_eq!(jvm.get_field::<i32>(&second, "keyCount", "I").await?, 1);
                assert_eq!(jvm.get_field::<i32>(&second, "notifyCount", "I").await?, 1);
                let popped: ClassInstanceRef<TestCard> = jvm.invoke_virtual(&canvas, "popCard", "()Lorg/kwis/msp/lcdui/Card;", ()).await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&popped, "equals", "(Ljava/lang/Object;)Z", (second.clone(),))
                        .await?
                );
                assert!(!jvm.invoke_virtual::<_, bool>(&second, "isShown", "()Z", ()).await?);
                assert_eq!(jvm.get_field::<i32>(&second, "hideCount", "I").await?, 1);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&canvas, "countCard", "()I", ()).await?, 1);

                assert!(
                    !jvm.invoke_virtual::<_, bool>(&canvas, "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", [None.into()])
                        .await?
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&canvas, "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", (second.clone(),),)
                        .await?
                );
                assert!(
                    jvm.invoke_virtual::<_, bool>(&canvas, "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", (first.clone(),))
                        .await?
                );
                assert!(!jvm.invoke_virtual::<_, bool>(&first, "isShown", "()Z", ()).await?);
                assert_eq!(jvm.get_field::<i32>(&first, "hideCount", "I").await?, 1);

                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (first.clone(),))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (second.clone(),))
                    .await?;
                let _: () = jvm.invoke_virtual(&canvas, "removeAllCards", "()V", ()).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&canvas, "countCard", "()I", ()).await?, 0);
                assert!(!jvm.invoke_virtual::<_, bool>(&first, "isShown", "()Z", ()).await?);
                assert!(!jvm.invoke_virtual::<_, bool>(&second, "isShown", "()Z", ()).await?);
                assert_eq!(jvm.get_field::<i32>(&first, "showCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&first, "hideCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&second, "showCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&second, "hideCount", "I").await?, 2);
                assert!(
                    jvm.invoke_virtual::<_, ClassInstanceRef<TestCard>>(&canvas, "popCard", "()Lorg/kwis/msp/lcdui/Card;", ())
                        .await?
                        .is_null()
                );

                Ok(())
            },
        )
    }

    #[test]
    fn test_card_repaint_translates_and_clamps() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestCard::as_proto(), TestCanvas::as_proto()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let display: ClassInstanceRef<Display> = jvm.instantiate_class("org/kwis/msp/lcdui/Display").await?.into();
                let card: ClassInstanceRef<TestCard> = jvm
                    .new_class("test/TestCard", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", (display, 10, 20, 30, 40, false))
                    .await?
                    .into();
                let canvas: ClassInstanceRef<TestCanvas> = jvm.new_class("test/TestCanvas", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(&card, "setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", (canvas.clone(),))
                    .await?;
                assert!(jvm.invoke_virtual::<_, bool>(&card, "isShown", "()Z", ()).await?);

                let _: () = jvm.invoke_virtual(&card, "repaint", "(IIII)V", (-5, 5, 20, 50)).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintX", "I").await?, 10);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintY", "I").await?, 25);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintWidth", "I").await?, 15);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintHeight", "I").await?, 35);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 1);

                let _: () = jvm.invoke_virtual(&card, "repaint", "(IIII)V", (40, 0, 5, 5)).await?;
                let _: () = jvm.invoke_virtual(&card, "repaint", "(IIII)V", (0, 0, -1, 5)).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 1);

                let _: () = jvm.invoke_virtual(&card, "repaint", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintX", "I").await?, 10);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintY", "I").await?, 20);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintWidth", "I").await?, 30);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintHeight", "I").await?, 40);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 2);

                let _: () = jvm
                    .invoke_virtual(&card, "setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", (None,))
                    .await?;
                let _: () = jvm.invoke_virtual(&card, "repaint", "()V", ()).await?;
                assert!(!jvm.invoke_virtual::<_, bool>(&card, "isShown", "()Z", ()).await?);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 2);

                Ok(())
            },
        )
    }

    #[test]
    fn test_card_canvas_offsets_and_isolates_graphics_state() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestCard::as_proto(), TestCanvas::as_proto()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let midp_display: ClassInstanceRef<MidpDisplay> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &midp_display,
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (canvas.clone(),),
                    )
                    .await?;
                let display: ClassInstanceRef<Display> = jvm.instantiate_class("org/kwis/msp/lcdui/Display").await?.into();
                let mutating_card: ClassInstanceRef<TestCard> = jvm
                    .new_class(
                        "test/TestCard",
                        "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V",
                        (display.clone(), 1, 1, 2, 2, true),
                    )
                    .await?
                    .into();
                let drawing_card: ClassInstanceRef<TestCard> = jvm
                    .new_class("test/TestCard", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", (display, 5, 2, 2, 3, false))
                    .await?
                    .into();
                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (mutating_card,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (drawing_card,))
                    .await?;

                let image: ClassInstanceRef<MidpImage> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (10, 10),
                    )
                    .await?;
                let graphics: ClassInstanceRef<MidpGraphics> = jvm
                    .invoke_virtual(&image, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
                    .await?;
                let _: () = jvm.invoke_virtual(&graphics, "setColor", "(I)V", (0x0000ff,)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (4, 4)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (-2, -3, 4, 3)).await?;

                let _: () = jvm
                    .invoke_virtual(&canvas, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics.clone(),))
                    .await?;

                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getTranslateX", "()I", ()).await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getTranslateY", "()I", ()).await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipX", "()I", ()).await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipY", "()I", ()).await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipWidth", "()I", ()).await?, 10);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getClipHeight", "()I", ()).await?, 10);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&graphics, "getColor", "()I", ()).await?, 0);

                let backend_image = MidpImage::image(&jvm, &image).await?;
                let outside = backend_image.get_pixel(0, 0);
                let first_outside = backend_image.get_pixel(1, 1);
                let first_dirty = backend_image.get_pixel(2, 1);
                let second_dirty = backend_image.get_pixel(5, 2);
                let second_outside = backend_image.get_pixel(6, 2);
                let below_dirty = backend_image.get_pixel(5, 4);
                assert_eq!((outside.r, outside.g, outside.b), (0, 0, 0));
                assert_eq!((first_outside.r, first_outside.g, first_outside.b), (0xff, 0, 0));
                assert_eq!((first_dirty.r, first_dirty.g, first_dirty.b), (0xff, 0, 0));
                assert_eq!((second_dirty.r, second_dirty.g, second_dirty.b), (0, 0xff, 0));
                assert_eq!((second_outside.r, second_outside.g, second_outside.b), (0, 0xff, 0));
                assert_eq!((below_dirty.r, below_dirty.g, below_dirty.b), (0, 0xff, 0));

                Ok(())
            },
        )
    }
}
