use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::{
    lcdui::{Display, Font, Graphics},
    midlet::MIDlet,
};

const TITLE_HORIZONTAL_PADDING: i32 = 4;
const TICKER_SCROLL_STEP: i32 = 2;
const TICKER_BACKGROUND: i32 = 0xdde5ec;
const BLACK: i32 = 0;
const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.Ticker
pub struct Ticker;

impl Ticker {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Ticker",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("advance", "(I)Z", Self::advance, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;III)V",
                    Self::paint,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("scrollOffset", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Ticker::<init>({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Ticker text is null").await);
        }

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "scrollOffset", "I", 0).await?;

        Ok(())
    }

    async fn advance(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<bool> {
        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let text = JavaLangString::to_rust_string(jvm, &text)
            .await?
            .replace("\r\n", " ")
            .replace(['\r', '\n'], " ");
        if text.is_empty() {
            return Ok(false);
        }
        let offset: i32 = jvm.get_field(&this, "scrollOffset", "I").await?;
        let offset = offset + TICKER_SCROLL_STEP;
        let offset = if offset >= Font::text_width(context.system().platform().font(), &text) + TITLE_HORIZONTAL_PADDING {
            TITLE_HORIZONTAL_PADDING - width
        } else {
            offset
        };
        jvm.put_field(&mut this, "scrollOffset", "I", offset).await?;
        Ok(true)
    }

    async fn paint(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        width: i32,
        y: i32,
        height: i32,
    ) -> JvmResult<()> {
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setClip",
                "(IIII)V",
                (0, y, width, height),
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TICKER_BACKGROUND,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (0, y, width, height),
            )
            .await?;

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let text = JavaLangString::to_rust_string(jvm, &text)
            .await?
            .replace("\r\n", " ")
            .replace(['\r', '\n'], " ");
        let text = JavaLangString::from_rust_string(jvm, &text).await?;
        let scroll_offset: i32 = jvm.get_field(&this, "scrollOffset", "I").await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (BLACK,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawString",
                "(Ljava/lang/String;III)V",
                (text, TITLE_HORIZONTAL_PADDING - scroll_offset, y + 2, LEFT_TOP),
            )
            .await?;
        Ok(())
    }

    async fn get_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Ticker::getString({this:?})");

        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn set_string(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Ticker::setString({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Ticker text is null").await);
        }

        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "scrollOffset", "I", 0).await?;

        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        if !midlet.is_null() {
            let display: ClassInstanceRef<Display> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Display",
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    (midlet,),
                )
                .await?;
            let ticker: ClassInstanceRef<Ticker> = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "getVisibleTicker",
                    "()Ljavax/microedition/lcdui/Ticker;",
                    (),
                )
                .await?;
            if !ticker.is_null() && ticker.identity() == this.identity() {
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "tickerChanged", "()V", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, -1, -1))
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use test_utils::{TestClock, TestPlatform, run_jvm_test_with_system};

    use wie_backend::{Event, System};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::{
            lcdui::{Display, Displayable, Image, Ticker},
            midlet::MIDlet,
        },
        get_protos,
    };

    const DISPLAY: &str = "javax/microedition/lcdui/Display";
    const DISPLAYABLE: &str = "javax/microedition/lcdui/Displayable";
    const TICKER: &str = "javax/microedition/lcdui/Ticker";
    const FORM: &str = "javax/microedition/lcdui/Form";

    struct TickerMidlet;

    impl TickerMidlet {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TickerMidlet",
                parent_class: Some("javax/microedition/midlet/MIDlet"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("startApp", "()V", Self::start_app, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await
        }

        async fn start_app(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<()> {
            Ok(())
        }
    }

    async fn pump(jvm: &Jvm, system: &System, clock: &TestClock, millis: u64) -> JvmResult<()> {
        clock.advance(millis);
        system.event_queue().push(Event::Redraw);
        let queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
        jvm.invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,)).await
    }

    async fn pixels(jvm: &Jvm, display: &ClassInstanceRef<Display>) -> JvmResult<Vec<u8>> {
        let _: () = jvm.invoke_virtual(display, DISPLAY, "handlePaintEvent", "()V", ()).await?;
        let image: ClassInstanceRef<Image> = jvm.get_field(display, "screenImage", "Ljavax/microedition/lcdui/Image;").await?;
        Ok(Image::image(jvm, &image).await?.raw().into_owned())
    }

    #[test]
    fn ticker_scrolls_on_the_current_screen_without_retaining_previous_screens() -> Result<()> {
        let clock = TestClock::new();
        run_jvm_test_with_system(
            Box::new([get_protos().into(), [TickerMidlet::as_proto()].into()]),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                jvm.push_native_frame();
                let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("test/TickerMidlet", "()V", ()).await?.into();
                let display = MIDlet::display(&jvm, &midlet).await?;
                let text = JavaLangString::from_rust_string(&jvm, "News\r\nNext\nEnd\r!").await?;
                let ticker: ClassInstanceRef<Ticker> = jvm.new_class(TICKER, "(Ljava/lang/String;)V", (text,)).await?.into();
                let ticker_root = jvm.new_global_ref(&ticker).unwrap();
                let first: ClassInstanceRef<Displayable> = jvm.new_class(FORM, "(Ljava/lang/String;)V", (None,)).await?.into();
                let first_root = jvm.new_global_ref(&first).unwrap();
                let second: ClassInstanceRef<Displayable> = jvm.new_class(FORM, "(Ljava/lang/String;)V", (None,)).await?.into();
                let second_root = jvm.new_global_ref(&second).unwrap();
                for screen in [&first, &second] {
                    let _: () = jvm
                        .invoke_virtual(
                            screen,
                            DISPLAYABLE,
                            "setTicker",
                            "(Ljavax/microedition/lcdui/Ticker;)V",
                            (ticker.clone(),),
                        )
                        .await?;
                }
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        DISPLAY,
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (first.clone(),),
                    )
                    .await?;
                let initial = pixels(&jvm, &display).await?;
                pump(&jvm, &system, &clock, 101).await?;
                let step: i32 = jvm.get_field(&ticker, "scrollOffset", "I").await?;
                assert!(step > 0, "a due event must move the ticker");
                assert_ne!(initial, pixels(&jvm, &display).await?);

                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        DISPLAY,
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (second.clone(),),
                    )
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, step);
                pump(&jvm, &system, &clock, 101).await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, step * 2);
                jvm.pop_frame();
                jvm.collect_garbage()?;
                drop(first_root);
                assert!(
                    jvm.collect_garbage()? > 0,
                    "the shared ticker and pending timers must release the old Form"
                );

                jvm.push_native_frame();
                let original = jvm.invoke_virtual(&ticker, TICKER, "getString", "()Ljava/lang/String;", ()).await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &original).await?, "News\r\nNext\nEnd\r!");
                let normalized = JavaLangString::from_rust_string(&jvm, "News Next End !").await?;
                let _: () = jvm
                    .invoke_virtual(&ticker, TICKER, "setString", "(Ljava/lang/String;)V", (normalized,))
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, 0);
                assert_eq!(initial, pixels(&jvm, &display).await?, "line endings must render as separators");
                pump(&jvm, &system, &clock, 101).await?;
                let replacement = JavaLangString::from_rust_string(&jvm, "Fresh").await?;
                let _: () = jvm
                    .invoke_virtual(&ticker, TICKER, "setString", "(Ljava/lang/String;)V", (replacement,))
                    .await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, 0);
                assert_ne!(initial, pixels(&jvm, &display).await?);
                pump(&jvm, &system, &clock, 101).await?;
                assert_eq!(
                    jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?,
                    step,
                    "old text timers must be stale"
                );

                let mut wrapped = false;
                let mut offset = step;
                for _ in 0..200 {
                    pump(&jvm, &system, &clock, 101).await?;
                    let next: i32 = jvm.get_field(&ticker, "scrollOffset", "I").await?;
                    wrapped |= next < offset;
                    offset = next;
                    if wrapped && offset >= 0 {
                        break;
                    }
                }
                assert!(wrapped && offset >= 0, "text must reenter from the right and keep scrolling");
                let _: () = jvm
                    .invoke_virtual(&second, DISPLAYABLE, "setTicker", "(Ljavax/microedition/lcdui/Ticker;)V", (None,))
                    .await?;
                pump(&jvm, &system, &clock, 101).await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, offset);
                assert!(system.event_queue().pop().is_none(), "detaching must stop recurring ticks");

                let _: () = jvm
                    .invoke_virtual(
                        &second,
                        DISPLAYABLE,
                        "setTicker",
                        "(Ljavax/microedition/lcdui/Ticker;)V",
                        (ticker.clone(),),
                    )
                    .await?;
                let hidden = jvm.new_class(FORM, "(Ljava/lang/String;)V", (None,)).await?;
                let _: () = jvm
                    .invoke_virtual(&display, DISPLAY, "setCurrent", "(Ljavax/microedition/lcdui/Displayable;)V", (hidden,))
                    .await?;
                pump(&jvm, &system, &clock, 101).await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, offset);
                let hidden_text = JavaLangString::from_rust_string(&jvm, "Hidden").await?;
                let _: () = jvm
                    .invoke_virtual(&ticker, TICKER, "setString", "(Ljava/lang/String;)V", (hidden_text,))
                    .await?;
                assert!(system.event_queue().pop().is_none(), "changing a hidden ticker must not schedule work");

                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        DISPLAY,
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Displayable;)V",
                        (second.clone(),),
                    )
                    .await?;
                let empty = JavaLangString::from_rust_string(&jvm, "").await?;
                let _: () = jvm
                    .invoke_virtual(&ticker, TICKER, "setString", "(Ljava/lang/String;)V", (empty,))
                    .await?;
                pump(&jvm, &system, &clock, 101).await?;
                assert_eq!(jvm.get_field::<i32>(&ticker, "scrollOffset", "I").await?, 0);
                assert!(system.event_queue().pop().is_none(), "empty text must not keep a timer alive");
                jvm.pop_frame();
                drop((ticker_root, second_root));
                Ok(())
            },
        )
    }
}
