use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::{Runnable, String};

use wie_backend::Event;
use wie_jvm_support::{JvmSupport, WieJvmContext};

use wie_midp::classes::{javax::microedition::lcdui::Display as MidpDisplay, net::wie::EventQueue as WieEventQueue};

use crate::classes::{
    net::wie::{CardCanvas, WIPIKeyCode},
    org::kwis::msp::lcdui::{Card, DisplayProxy, Jlet, JletEventListener},
};

// class org.kwis.msp.lcdui.Display
#[java_class(name = "org/kwis/msp/lcdui/Display", public)]
pub struct Display {
    #[field(private)]
    midpDisplay: ClassInstanceRef<MidpDisplay>,
    #[field(private)]
    cardCanvas: ClassInstanceRef<CardCanvas>,
    #[field(private)]
    dockedCard: ClassInstanceRef<Card>,
}

#[java_methods]
impl Display {
    #[method(constructor)]
    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
        display_proxy: ClassInstanceRef<DisplayProxy>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::<init>({this:?}, {jlet:?}, {display_proxy:?})");

        let midlet = Jlet::midlet(jvm, &jlet).await?;

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getDisplay",
                "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                (midlet,),
            )
            .await?;

        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/Display",
            "midpDisplay",
            "Ljavax/microedition/lcdui/Display;",
            midp_display.clone(),
        )
        .await?;

        let card_canvas = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/Display",
            "cardCanvas",
            "Lnet/wie/CardCanvas;",
            card_canvas.clone(),
        )
        .await?;

        let _: () = jvm
            .invoke_virtual(
                &midp_display,
                "javax/microedition/lcdui/Display",
                "setCurrent",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (card_canvas,),
            )
            .await?;

        Ok(())
    }

    #[method(public, static, final)]
    async fn get_display(jvm: &Jvm, _: &mut WieJvmContext, str: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({str:?})");

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = Jlet::display(jvm, &jlet).await?;

        Ok(display)
    }

    #[method(public, static)]
    async fn get_default_display(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDefaultDisplay");

        let result = jvm
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                [None.into()],
            )
            .await?;

        Ok(result)
    }

    #[method(public)]
    async fn get_docked_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDockedCard({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lcdui/Display", "dockedCard", "Lorg/kwis/msp/lcdui/Card;")
            .await
    }

    #[method(public)]
    async fn set_docked_card(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        card: ClassInstanceRef<Card>,
        where_: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::setDockedCard({this:?}, {card:?}, {where_})");

        jvm.put_field(&mut this, "org/kwis/msp/lcdui/Display", "dockedCard", "Lorg/kwis/msp/lcdui/Card;", card)
            .await
    }

    #[method(public)]
    async fn is_double_buffered(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Display::isDoubleBuffered({this:?})");

        let canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;

        jvm.invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "isDoubleBuffered", "()Z", ())
            .await
    }

    #[method(public, final)]
    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({this:?}, {c:?})");

        let card_canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;
        let _: () = jvm
            .invoke_virtual(&card_canvas, "net/wie/CardCanvas", "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (c,))
            .await?;

        Ok(())
    }

    #[method(public, final)]
    async fn pop_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::popCard({this:?})");

        let card_canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;
        jvm.invoke_virtual(&card_canvas, "net/wie/CardCanvas", "popCard", "()Lorg/kwis/msp/lcdui/Card;", ())
            .await
    }

    #[method(public, final)]
    async fn remove_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, card: ClassInstanceRef<Card>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeCard({this:?}, {card:?})");

        let card_canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;
        jvm.invoke_virtual(&card_canvas, "net/wie/CardCanvas", "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", (card,))
            .await
    }

    #[method(public, final)]
    async fn count_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::countCard({this:?})");

        let card_canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;
        jvm.invoke_virtual(&card_canvas, "net/wie/CardCanvas", "countCard", "()I", ()).await
    }

    #[method(public)]
    async fn remove_all_cards(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards({this:?})");

        let card_canvas: ClassInstanceRef<CardCanvas> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "cardCanvas", "Lnet/wie/CardCanvas;")
            .await?;
        let _: () = jvm
            .invoke_virtual(&card_canvas, "net/wie/CardCanvas", "removeAllCards", "()V", ())
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn add_jlet_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Display>,
        qel: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({this:?}, {qel:?})");

        Ok(())
    }

    #[method(public, final)]
    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({this:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "midpDisplay", "Ljavax/microedition/lcdui/Display;")
            .await?;
        let width: i32 = jvm
            .invoke_virtual(&midp_display, "javax/microedition/lcdui/Display", "getWidth", "()I", ())
            .await?;

        Ok(width)
    }

    #[method(public, final)]
    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({this:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "midpDisplay", "Ljavax/microedition/lcdui/Display;")
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&midp_display, "javax/microedition/lcdui/Display", "getHeight", "()I", ())
            .await?;

        Ok(height)
    }

    #[method(public, final)]
    async fn call_serially(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r: ClassInstanceRef<Runnable>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({this:?}, {r:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/Display", "midpDisplay", "Ljavax/microedition/lcdui/Display;")
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &midp_display,
                "javax/microedition/lcdui/Display",
                "callSerially",
                "(Ljava/lang/Runnable;)V",
                (r,),
            )
            .await?;

        Ok(())
    }

    #[method(name = "callSerially", public, final)]
    async fn call_serially_with_timeout(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        runnable: ClassInstanceRef<Runnable>,
        timeout: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({this:?}, {runnable:?}, {timeout})");

        let Some(runnable) = jvm.new_global_ref(&runnable) else {
            return Err(jvm.exception("java/lang/NullPointerException", "Runnable is null").await);
        };
        let due = context.system().platform().now() + timeout.max(0) as u64;
        let jvm = jvm.clone();
        context.system().event_queue().push(Event::timer(due, move || async move {
            let result: JvmResult<()> = async {
                let queue: ClassInstanceRef<WieEventQueue> = jvm
                    .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                    .await?;
                jvm.invoke_virtual(
                    &queue,
                    "net/wie/EventQueue",
                    "callSerially",
                    "(Ljava/lang/Runnable;)V",
                    ((*runnable).clone(),),
                )
                .await
            }
            .await;
            match result {
                Ok(()) => Ok(()),
                Err(error) => Err(JvmSupport::to_wie_err(&jvm, error).await),
            }
        }));

        Ok(())
    }

    #[method(public, final)]
    async fn is_color(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::isColor({this:?})");

        Ok(false)
    }

    #[method(public, final)]
    async fn num_colors(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::numColors({this:?})");

        Ok(0)
    }

    #[method(public, final)]
    async fn has_pointer_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasPointerEvents({this:?})");

        Ok(false)
    }

    #[method(public, final)]
    async fn has_pointer_motion_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasPointerMotionEvents({this:?})");

        Ok(false)
    }

    #[method(public, final)]
    async fn has_repeat_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasRepeatEvents({this:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn get_key_name(_: &Jvm, _: &mut WieJvmContext, key: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getKeyName({key})");

        Ok(None.into())
    }

    #[method(public)]
    async fn get_bits_per_pixel(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getBitsPerPixel({this:?})");

        Ok(0)
    }

    #[method(public)]
    async fn flush(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::flush({this:?})");

        Ok(())
    }

    #[method(public)]
    async fn remove_jlet_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::removeJletEventListener({this:?}, {listener:?})");

        Ok(())
    }

    #[method(public)]
    async fn grab_key(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        key: i32,
        listener: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::grabKey({this:?}, {key}, {listener:?})");

        Ok(())
    }

    #[method(public)]
    async fn ungrab_key(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::ungrabKey({this:?}, {key})");

        Ok(())
    }

    #[method(public, native, static)]
    async fn get_game_action(_jvm: &Jvm, _: &mut WieJvmContext, key: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getGameAction({key})");

        let action = match WIPIKeyCode::from_raw(key) {
            Some(WIPIKeyCode::UP) => 1,
            Some(WIPIKeyCode::DOWN) => 6,
            Some(WIPIKeyCode::LEFT) => 2,
            Some(WIPIKeyCode::RIGHT) => 5,
            Some(WIPIKeyCode::FIRE) => 8,
            Some(WIPIKeyCode::CLEAR) => 99,
            _ => key,
        };

        Ok(action)
    }

    #[method(public, native, static)]
    async fn get_key_code(_jvm: &Jvm, _: &mut WieJvmContext, game_key: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getKeyCode({game_key})");

        let key_code = match game_key {
            1 => WIPIKeyCode::UP as i32,
            2 => WIPIKeyCode::LEFT as i32,
            5 => WIPIKeyCode::RIGHT as i32,
            6 => WIPIKeyCode::DOWN as i32,
            8 => WIPIKeyCode::FIRE as i32,
            90 => WIPIKeyCode::LEFT_SOFT_KEY as i32,
            91 => WIPIKeyCode::RIGHT_SOFT_KEY as i32,
            92 => -8,
            96 => WIPIKeyCode::VOLUME_UP as i32,
            97 => WIPIKeyCode::VOLUME_DOWN as i32,
            98 => -15,
            99 => WIPIKeyCode::CLEAR as i32,
            _ => 0,
        };

        Ok(key_code)
    }

    pub async fn midp_display(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<MidpDisplay>> {
        jvm.get_field(this, "org/kwis/msp/lcdui/Display", "midpDisplay", "Ljavax/microedition/lcdui/Display;")
            .await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult};
    use jvm_class_proto::{java_class, java_methods};

    use test_utils::{TestClock, TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::{Event, KeyCode};
    use wie_jvm_support::WieJvmContext;
    use wie_midp::classes::net::wie::EventQueue;
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::lcdui::{Display, Jlet},
        get_protos,
    };

    #[java_class(name = "test/DelayedCallback", implements("java/lang/Runnable"), public)]
    struct DelayedCallback {
        #[field(private)]
        id: i32,
        #[field(static)]
        calls: i32,
    }

    #[java_methods]
    impl DelayedCallback {
        #[method(public)]
        async fn run(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let id: i32 = jvm.get_field(&this, "test/DelayedCallback", "id", "I").await?;
            let calls: i32 = jvm.get_static_field("test/DelayedCallback", "calls", "I").await?;
            jvm.put_static_field("test/DelayedCallback", "calls", "I", calls * 10 + id).await
        }
    }

    #[java_class(name = "test/DelayedJlet", extends = "org/kwis/msp/lcdui/Jlet", public)]
    struct DelayedJlet;

    #[java_methods]
    impl DelayedJlet {}

    #[test]
    fn delayed_callbacks_follow_deadlines_and_survive_gc() -> Result<()> {
        let clock = TestClock::new();
        run_jvm_test_with_system(
            Box::new([
                wie_midp::get_protos().into(),
                get_protos().into(),
                Box::new([DelayedCallback::as_proto(), DelayedJlet::as_proto()]),
            ]),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                let _midlet = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await?;
                let jlet = jvm.instantiate_class("test/DelayedJlet").await?;
                let _: () = jvm.invoke_special(&jlet, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await?;
                let current: ClassInstanceRef<Jlet> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Jlet", "getCurrentJlet", "()Lorg/kwis/msp/lcdui/Jlet;", ())
                    .await?;
                let is_current: bool = jvm
                    .invoke_virtual(&jlet, "java/lang/Object", "equals", "(Ljava/lang/Object;)Z", (current,))
                    .await?;
                assert!(is_current);
                let display: ClassInstanceRef<Display> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                    .await?;
                let queue: ClassInstanceRef<EventQueue> = jvm
                    .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
                    .await?;
                let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
                for (id, timeout) in [(1, 80), (2, 20), (3, -1)] {
                    let frame = jvm.push_native_frame(&[]);
                    let mut callback: ClassInstanceRef<DelayedCallback> = jvm.instantiate_class("test/DelayedCallback").await?.into();
                    jvm.put_field(&mut callback, "test/DelayedCallback", "id", "I", id).await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "org/kwis/msp/lcdui/Display",
                            "callSerially",
                            "(Ljava/lang/Runnable;I)V",
                            (callback, timeout),
                        )
                        .await?;
                    drop(frame);
                }
                jvm.collect_garbage().unwrap();
                assert_eq!(jvm.get_static_field::<i32>("test/DelayedCallback", "calls", "I").await?, 0);
                for (time, expected) in [(0, 3), (19, 3), (20, 32), (79, 32), (80, 321), (100, 321)] {
                    clock.set(time);
                    system.event_queue().push(Event::Keydown(KeyCode::NUM1));
                    system.event_queue().push(Event::Redraw);
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(jvm.load_array::<i32>(&event, 0, 1).await?, [1], "input must precede due callbacks");
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(&queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                        .await?;
                    assert_eq!(jvm.get_static_field::<i32>("test/DelayedCallback", "calls", "I").await?, expected);
                }
                let error = jvm
                    .invoke_virtual::<_, ()>(
                        &display,
                        "org/kwis/msp/lcdui/Display",
                        "callSerially",
                        "(Ljava/lang/Runnable;I)V",
                        (None, 1),
                    )
                    .await
                    .unwrap_err();
                let JavaError::JavaException(exception) = error;
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
                Ok(())
            },
        )
    }

    #[test]
    fn test_get_key_code() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let up: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (1,)).await?;
            let down: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (6,)).await?;
            let left: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (2,)).await?;
            let right: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (5,)).await?;
            let fire: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (8,)).await?;
            let soft1: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (90,)).await?;
            let soft2: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (91,)).await?;
            let soft3: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (92,)).await?;
            let side_up: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (96,)).await?;
            let side_down: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (97,)).await?;
            let side_select: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (98,)).await?;
            let clear: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (99,)).await?;
            let game_a: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (9,)).await?;
            let invalid: i32 = jvm.invoke_static("org/kwis/msp/lcdui/Display", "getKeyCode", "(I)I", (1234,)).await?;

            assert_eq!(up, -1);
            assert_eq!(down, -2);
            assert_eq!(left, -3);
            assert_eq!(right, -4);
            assert_eq!(fire, -5);
            assert_eq!(soft1, -6);
            assert_eq!(soft2, -7);
            assert_eq!(soft3, -8);
            assert_eq!(side_up, -13);
            assert_eq!(side_down, -14);
            assert_eq!(side_select, -15);
            assert_eq!(clear, -16);
            assert_eq!(game_a, 0);
            assert_eq!(invalid, 0);

            Ok(())
        })
    }
}
