use alloc::format;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::Canvas;

use crate::classes::org::kwis::msp::lcdui::{Display, Graphics};

// abstract class org.kwis.msp.lcdui.Card
#[java_class(name = "org/kwis/msp/lcdui/Card", implements("org/kwis/msp/lcdui/JletEventListener"), public, abstract)]
pub struct Card {
    #[field(private)]
    canvas: ClassInstanceRef<Canvas>,
    #[field(private)]
    display: ClassInstanceRef<Display>,
    #[field(protected)]
    x: i32,
    #[field(protected)]
    y: i32,
    #[field(protected)]
    w: i32,
    #[field(protected)]
    h: i32,
    #[field(protected)]
    transparent: bool,
}

#[java_methods]
impl Card {
    #[method(constructor, public)]
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
    #[method(constructor, public)]
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

    #[method(constructor, public)]
    async fn init_transparent(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, transparent: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::<init>({this:?}, {transparent})");

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        if display.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "display is null").await);
        }

        let width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", []).await?;
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

    #[method(constructor, public)]
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

    #[method(constructor, public)]
    async fn init_with_display(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, display: ClassInstanceRef<Display>) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Card::<init>({this:?}, {display:?})");
        tracing::debug!("{log}");

        if display.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "display is null").await);
        }

        let width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", []).await?;
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

    #[method(constructor, public)]
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

    #[method(constructor, public)]
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
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "display", "Lorg/kwis/msp/lcdui/Display;", display)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "x", "I", x).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "y", "I", y).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "w", "I", width).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "h", "I", height).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "transparent", "Z", transparent)
            .await?;

        Ok(())
    }

    #[method(name = "move", public)]
    async fn move_card(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::move({this:?}, {x}, {y})");

        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "x", "I", x).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "y", "I", y).await
    }

    #[method(public)]
    async fn resize(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::resize({this:?}, {width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "w", "I", width).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Card", "h", "I", height).await
    }

    #[method(public)]
    async fn get_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getX({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "x", "I").await
    }

    #[method(public)]
    async fn get_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getY({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "y", "I").await
    }

    #[method(protected)]
    async fn pointer_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, r#type: i32, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::pointerNotify({this:?}, {type}, {x}, {y})");

        Ok(false)
    }

    #[method(public)]
    async fn get_display(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<ClassInstanceRef<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getDisplay({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "display", "Lorg/kwis/msp/lcdui/Display;")
            .await
    }

    #[method(public)]
    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Card::isShown({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Card", "canvas", "Ljavax/microedition/lcdui/Canvas;")
            .await?;
        Ok(!canvas.is_null())
    }

    #[method(public)]
    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "w", "I").await
    }

    #[method(public)]
    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "h", "I").await
    }

    #[method(public)]
    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({this:?})");

        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "h", "I").await?;

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (0, 0, width, height))
            .await?;

        Ok(())
    }

    #[method(name = "repaint", public)]
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

        let canvas: ClassInstanceRef<Canvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Card", "canvas", "Ljavax/microedition/lcdui/Canvas;")
            .await?;
        if canvas.is_null() || width <= 0 || height <= 0 {
            return Ok(());
        }

        let card_x: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "x", "I").await?;
        let card_y: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "y", "I").await?;
        let card_width: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "w", "I").await?;
        let card_height: i32 = jvm.get_field(&this, "org/kwis/msp/lcdui/Card", "h", "I").await?;

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
                "javax/microedition/lcdui/Canvas",
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

    #[method(public)]
    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::serviceRepaints({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Card", "canvas", "Ljavax/microedition/lcdui/Canvas;")
            .await?;
        if !canvas.is_null() {
            let _: () = jvm
                .invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
                .await?;
        }

        Ok(())
    }

    #[method(protected)]
    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, show: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::showNotify({this:?}, {show})");

        Ok(())
    }

    #[method(protected)]
    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Card>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Card::keyNotify({this:?}, {type}, {key})");

        Ok(false)
    }

    // wie private
    #[method(private)]
    async fn set_canvas(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Card>, canvas: ClassInstanceRef<Canvas>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::setCanvas({this:?}, {canvas:?})");

        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/Card",
            "canvas",
            "Ljavax/microedition/lcdui/Canvas;",
            canvas,
        )
        .await
    }

    #[method(protected)]
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()>;
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{java_class, java_methods};

    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::{Display as MidpDisplay, Graphics as MidpGraphics, Image as MidpImage};
    use wie_util::Result;

    use crate::{
        classes::net::wie::CardCanvas,
        classes::org::kwis::msp::lcdui::{Display, Graphics},
        get_protos,
    };

    #[java_class(name = "test/TestCard", extends = "org/kwis/msp/lcdui/Card", public)]
    struct TestCard {
        #[field(private)]
        showCount: i32,
        #[field(private)]
        hideCount: i32,
        #[field(private)]
        keyCount: i32,
        #[field(private)]
        notifyCount: i32,
        #[field(private)]
        paintCount: i32,
    }

    #[java_methods]
    impl TestCard {
        #[method(constructor, public)]
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

        #[method(constructor, public)]
        async fn init_with_display(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            this: ClassInstanceRef<Self>,
            display: ClassInstanceRef<Display>,
        ) -> JvmResult<()> {
            jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
                .await
        }

        #[method(protected)]
        async fn paint(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
            let paint_count: i32 = jvm.get_field(&this, "test/TestCard", "paintCount", "I").await?;
            jvm.put_field(&mut this, "test/TestCard", "paintCount", "I", paint_count + 1).await?;

            let width: i32 = jvm.get_field(&this, "test/TestCard", "w", "I").await?;
            let height: i32 = jvm.get_field(&this, "test/TestCard", "h", "I").await?;
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "org/kwis/msp/lcdui/Graphics", "getColor", "()I", ())
                    .await?,
                0
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "org/kwis/msp/lcdui/Graphics", "getAlpha", "()I", ())
                    .await?,
                255
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "org/kwis/msp/lcdui/Graphics", "getStrokeStyle", "()I", ())
                    .await?,
                0
            );
            assert!(
                !jvm.invoke_virtual::<_, bool>(&graphics, "org/kwis/msp/lcdui/Graphics", "isXORMode", "()Z", ())
                    .await?
            );

            let transparent: bool = jvm.get_field(&this, "test/TestCard", "transparent", "Z").await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "org/kwis/msp/lcdui/Graphics",
                    "setColor",
                    "(I)V",
                    (if transparent { 0xff0000 } else { 0x00ff00 },),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "org/kwis/msp/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (-1, -1, width + 2, height + 2),
                )
                .await?;

            if transparent {
                let _: () = jvm
                    .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "translate", "(II)V", (7, 8))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 1, 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setAlpha", "(I)V", (0,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setStrokeStyle", "(I)V", (1,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setXORMode", "(Z)V", (true,))
                    .await?;
            }

            Ok(())
        }

        #[method(protected)]
        async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
            let _: () = jvm
                .invoke_special(&this, "org/kwis/msp/lcdui/Card", "showNotify", "(Z)V", (show,))
                .await?;
            let field = if show { "showCount" } else { "hideCount" };
            let count: i32 = jvm.get_field(&this, "test/TestCard", field, "I").await?;
            jvm.put_field(&mut this, "test/TestCard", field, "I", count + 1).await
        }

        #[method(protected)]
        async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: i32, _: i32) -> JvmResult<bool> {
            let count: i32 = jvm.get_field(&this, "test/TestCard", "keyCount", "I").await?;
            jvm.put_field(&mut this, "test/TestCard", "keyCount", "I", count + 1).await?;
            Ok(false)
        }

        #[method(public)]
        async fn notify_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: i32, _: i32, _: i32) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "test/TestCard", "notifyCount", "I").await?;
            jvm.put_field(&mut this, "test/TestCard", "notifyCount", "I", count + 1).await
        }
    }

    #[java_class(name = "test/TestCanvas", extends = "javax/microedition/lcdui/Canvas", public)]
    struct TestCanvas {
        #[field(private)]
        repaintX: i32,
        #[field(private)]
        repaintY: i32,
        #[field(private)]
        repaintWidth: i32,
        #[field(private)]
        repaintHeight: i32,
        #[field(private)]
        repaintCount: i32,
    }

    #[java_methods]
    impl TestCanvas {
        #[method(constructor, public)]
        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        #[method(protected)]
        async fn paint(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, _: ClassInstanceRef<MidpGraphics>) -> JvmResult<()> {
            Ok(())
        }

        #[method(public)]
        async fn repaint(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "test/TestCanvas", "repaintCount", "I").await?;
            jvm.put_field(&mut this, "test/TestCanvas", "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "test/TestCanvas", "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "test/TestCanvas", "repaintWidth", "I", width).await?;
            jvm.put_field(&mut this, "test/TestCanvas", "repaintHeight", "I", height).await?;
            jvm.put_field(&mut this, "test/TestCanvas", "repaintCount", "I", count + 1).await
        }
    }

    #[test]
    fn test_card_state_and_validation() -> Result<()> {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [TestCard::as_proto()].into()]),
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

                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getX", "()I", ()).await?,
                    3
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getY", "()I", ()).await?,
                    4
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ())
                        .await?,
                    20
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getHeight", "()I", ())
                        .await?,
                    30
                );

                let returned_display: ClassInstanceRef<Display> = jvm
                    .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "getDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                    .await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(
                        &returned_display,
                        "org/kwis/msp/lcdui/Display",
                        "equals",
                        "(Ljava/lang/Object;)Z",
                        (display.clone(),)
                    )
                    .await?
                );

                let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "move", "(II)V", (-5, 7)).await?;
                let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "resize", "(II)V", (40, 50)).await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getX", "()I", ()).await?,
                    -5
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getY", "()I", ()).await?,
                    7
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ())
                        .await?,
                    40
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getHeight", "()I", ())
                        .await?,
                    50
                );

                assert!(
                    jvm.invoke_virtual::<_, ()>(&card, "org/kwis/msp/lcdui/Card", "resize", "(II)V", (0, 10))
                        .await
                        .is_err()
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ())
                        .await?,
                    40
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", "getHeight", "()I", ())
                        .await?,
                    50
                );

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
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [TestCard::as_proto()].into()]),
            |jvm| async move {
                let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let midp_display: ClassInstanceRef<MidpDisplay> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &midp_display,
                        "javax/microedition/lcdui/Display",
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
                    .invoke_virtual(&canvas, "net/wie/CardCanvas", "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", [None.into()])
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "net/wie/CardCanvas", "countCard", "()I", ())
                        .await?,
                    0
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&first, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );

                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (first.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (first.clone(),),
                    )
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "net/wie/CardCanvas", "countCard", "()I", ())
                        .await?,
                    1
                );
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "showCount", "I").await?, 1);
                assert!(
                    jvm.invoke_virtual::<_, bool>(&first, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                let paint_count: i32 = jvm.get_field(&first, "test/TestCard", "paintCount", "I").await?;
                let _: () = jvm
                    .invoke_virtual(&first, "org/kwis/msp/lcdui/Card", "serviceRepaints", "()V", ())
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "paintCount", "I").await?, paint_count + 1);

                let other_canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &other_canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (first.clone(),),
                    )
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&other_canvas, "net/wie/CardCanvas", "countCard", "()I", ())
                        .await?,
                    0
                );

                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (second.clone(),),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&canvas, "net/wie/CardCanvas", "keyPressed", "(I)V", (42,)).await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "net/wie/CardCanvas", "handleNotifyEvent", "(III)V", (1, 2, 3))
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "keyCount", "I").await?, 0);
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "notifyCount", "I").await?, 0);
                assert_eq!(jvm.get_field::<i32>(&second, "test/TestCard", "keyCount", "I").await?, 1);
                assert_eq!(jvm.get_field::<i32>(&second, "test/TestCard", "notifyCount", "I").await?, 1);
                let popped: ClassInstanceRef<TestCard> = jvm
                    .invoke_virtual(&canvas, "net/wie/CardCanvas", "popCard", "()Lorg/kwis/msp/lcdui/Card;", ())
                    .await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&popped, "org/kwis/msp/lcdui/Card", "equals", "(Ljava/lang/Object;)Z", (second.clone(),))
                        .await?
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&second, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&second, "test/TestCard", "hideCount", "I").await?, 1);
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "net/wie/CardCanvas", "countCard", "()I", ())
                        .await?,
                    1
                );

                assert!(
                    !jvm.invoke_virtual::<_, bool>(&canvas, "net/wie/CardCanvas", "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", [None.into()])
                        .await?
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(
                        &canvas,
                        "net/wie/CardCanvas",
                        "removeCard",
                        "(Lorg/kwis/msp/lcdui/Card;)Z",
                        (second.clone(),),
                    )
                    .await?
                );
                assert!(
                    jvm.invoke_virtual::<_, bool>(
                        &canvas,
                        "net/wie/CardCanvas",
                        "removeCard",
                        "(Lorg/kwis/msp/lcdui/Card;)Z",
                        (first.clone(),)
                    )
                    .await?
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&first, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "hideCount", "I").await?, 1);

                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (first.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (second.clone(),),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&canvas, "net/wie/CardCanvas", "removeAllCards", "()V", ()).await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&canvas, "net/wie/CardCanvas", "countCard", "()I", ())
                        .await?,
                    0
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&first, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&second, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "showCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&first, "test/TestCard", "hideCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&second, "test/TestCard", "showCount", "I").await?, 2);
                assert_eq!(jvm.get_field::<i32>(&second, "test/TestCard", "hideCount", "I").await?, 2);
                assert!(
                    jvm.invoke_virtual::<_, ClassInstanceRef<TestCard>>(&canvas, "net/wie/CardCanvas", "popCard", "()Lorg/kwis/msp/lcdui/Card;", (),)
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
                    .invoke_virtual(
                        &card,
                        "org/kwis/msp/lcdui/Card",
                        "setCanvas",
                        "(Ljavax/microedition/lcdui/Canvas;)V",
                        (canvas.clone(),),
                    )
                    .await?;
                assert!(
                    jvm.invoke_virtual::<_, bool>(&card, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );

                let _: () = jvm
                    .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (-5, 5, 20, 50))
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintX", "I").await?, 10);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintY", "I").await?, 25);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintWidth", "I").await?, 15);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintHeight", "I").await?, 35);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintCount", "I").await?, 1);

                let _: () = jvm
                    .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (40, 0, 5, 5))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (0, 0, -1, 5))
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintCount", "I").await?, 1);

                let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintX", "I").await?, 10);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintY", "I").await?, 20);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintWidth", "I").await?, 30);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintHeight", "I").await?, 40);
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintCount", "I").await?, 2);

                let _: () = jvm
                    .invoke_virtual(
                        &card,
                        "org/kwis/msp/lcdui/Card",
                        "setCanvas",
                        "(Ljavax/microedition/lcdui/Canvas;)V",
                        (None,),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "()V", ()).await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&card, "org/kwis/msp/lcdui/Card", "isShown", "()Z", ())
                        .await?
                );
                assert_eq!(jvm.get_field::<i32>(&canvas, "test/TestCanvas", "repaintCount", "I").await?, 2);

                Ok(())
            },
        )
    }

    #[test]
    fn test_card_canvas_offsets_and_isolates_graphics_state() -> Result<()> {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [TestCard::as_proto()].into()]),
            |jvm| async move {
                let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?.into();
                let midp_display: ClassInstanceRef<MidpDisplay> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &midp_display,
                        "javax/microedition/lcdui/Display",
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
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "pushCard",
                        "(Lorg/kwis/msp/lcdui/Card;)V",
                        (mutating_card,),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&canvas, "net/wie/CardCanvas", "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (drawing_card,))
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
                    .invoke_virtual(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x0000ff,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (2, 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (-1, 0, 8, 4))
                    .await?;

                let _: () = jvm
                    .invoke_virtual(
                        &canvas,
                        "net/wie/CardCanvas",
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        (graphics.clone(),),
                    )
                    .await?;

                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateX", "()I", ())
                        .await?,
                    0
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateY", "()I", ())
                        .await?,
                    0
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                        .await?,
                    0
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                        .await?,
                    0
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                        .await?,
                    10
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                        .await?,
                    10
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getColor", "()I", ())
                        .await?,
                    0
                );

                let backend_image = MidpImage::image(&jvm, &image).await?;
                for (x, y, expected) in [
                    (0, 0, (0xff, 0xff, 0xff)),
                    (2, 2, (0xff, 0xff, 0xff)),
                    (3, 2, (0xff, 0, 0)),
                    (4, 3, (0xff, 0, 0)),
                    (7, 3, (0, 0xff, 0)),
                    (8, 4, (0, 0xff, 0)),
                    (9, 3, (0xff, 0xff, 0xff)),
                    (7, 5, (0xff, 0xff, 0xff)),
                ] {
                    let pixel = backend_image.get_pixel(x, y);
                    assert_eq!((pixel.r, pixel.g, pixel.b), expected, "pixel at ({x}, {y})");
                }

                Ok(())
            },
        )
    }
}
