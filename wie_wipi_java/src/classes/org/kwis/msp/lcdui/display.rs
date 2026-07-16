use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::{Object, Runnable, String};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use wie_midp::classes::javax::microedition::lcdui::Display as MidpDisplay;

use crate::classes::{
    net::wie::WIPIKeyCode,
    org::kwis::msp::lcdui::{Card, Jlet, JletEventListener},
};

// class org.kwis.msp.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                    Self::init,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("isDoubleBuffered", "()Z", Self::is_double_buffered, Default::default()),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card, Default::default()),
                JavaMethodProto::new(
                    "setDockedCard",
                    "(Lorg/kwis/msp/lcdui/Card;I)V",
                    Self::set_docked_card,
                    Default::default(),
                ),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, Default::default()),
                JavaMethodProto::new("popCard", "()Lorg/kwis/msp/lcdui/Card;", Self::pop_card, Default::default()),
                JavaMethodProto::new("removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", Self::remove_card, Default::default()),
                JavaMethodProto::new("countCard", "()I", Self::count_card, Default::default()),
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, Default::default()),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new(
                    "callSerially",
                    "(Ljava/lang/Runnable;I)V",
                    Self::call_serially_with_timeout,
                    Default::default(),
                ),
                JavaMethodProto::new("isColor", "()Z", Self::is_color, Default::default()),
                JavaMethodProto::new("numColors", "()I", Self::num_colors, Default::default()),
                JavaMethodProto::new("hasPointerEvents", "()Z", Self::has_pointer_events, Default::default()),
                JavaMethodProto::new("hasPointerMotionEvents", "()Z", Self::has_pointer_motion_events, Default::default()),
                JavaMethodProto::new("hasRepeatEvents", "()Z", Self::has_repeat_events, Default::default()),
                JavaMethodProto::new("getKeyName", "(I)Ljava/lang/String;", Self::get_key_name, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getBitsPerPixel", "()I", Self::get_bits_per_pixel, Default::default()),
                JavaMethodProto::new("flush", "()V", Self::flush, Default::default()),
                JavaMethodProto::new(
                    "removeJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::remove_jlet_event_listener,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "grabKey",
                    "(ILorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::grab_key,
                    Default::default(),
                ),
                JavaMethodProto::new("ungrabKey", "(I)V", Self::ungrab_key, Default::default()),
                JavaMethodProto::new(
                    "getGameAction",
                    "(I)I",
                    Self::get_game_action,
                    MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getKeyCode",
                    "(I)I",
                    Self::get_key_code,
                    MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("midpDisplay", "Ljavax/microedition/lcdui/Display;", Default::default()),
                JavaFieldProto::new("cardCanvas", "Lnet/wie/CardCanvas;", Default::default()),
                JavaFieldProto::new("dockedCard", "Lorg/kwis/msp/lcdui/Card;", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
        display_proxy: ClassInstanceRef<Object>,
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

        jvm.put_field(&mut this, "midpDisplay", "Ljavax/microedition/lcdui/Display;", midp_display.clone())
            .await?;

        let card_canvas = jvm.new_class("net/wie/CardCanvas", "()V", ()).await?;
        jvm.put_field(&mut this, "cardCanvas", "Lnet/wie/CardCanvas;", card_canvas.clone())
            .await?;

        let _: () = jvm
            .invoke_virtual(&midp_display, "setCurrent", "(Ljavax/microedition/lcdui/Displayable;)V", (card_canvas,))
            .await?;

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _: &mut WieJvmContext, str: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({str:?})");

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = Jlet::display(jvm, &jlet).await?;

        Ok(display)
    }

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

    async fn get_docked_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDockedCard({this:?})");

        jvm.get_field(&this, "dockedCard", "Lorg/kwis/msp/lcdui/Card;").await
    }

    async fn set_docked_card(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        card: ClassInstanceRef<Card>,
        where_: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::setDockedCard({this:?}, {card:?}, {where_})");

        jvm.put_field(&mut this, "dockedCard", "Lorg/kwis/msp/lcdui/Card;", card).await
    }

    async fn is_double_buffered(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Display::isDoubleBuffered({this:?})");

        let canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;

        jvm.invoke_virtual(&canvas, "isDoubleBuffered", "()Z", ()).await
    }

    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({this:?}, {c:?})");

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;
        let _: () = jvm.invoke_virtual(&card_canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (c,)).await?;

        Ok(())
    }

    async fn pop_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::popCard({this:?})");

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;
        jvm.invoke_virtual(&card_canvas, "popCard", "()Lorg/kwis/msp/lcdui/Card;", ()).await
    }

    async fn remove_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, card: ClassInstanceRef<Card>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeCard({this:?}, {card:?})");

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;
        jvm.invoke_virtual(&card_canvas, "removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", (card,))
            .await
    }

    async fn count_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::countCard({this:?})");

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;
        jvm.invoke_virtual(&card_canvas, "countCard", "()I", ()).await
    }

    async fn remove_all_cards(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards({this:?})");

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lnet/wie/CardCanvas;").await?;
        let _: () = jvm.invoke_virtual(&card_canvas, "removeAllCards", "()V", ()).await?;

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Display>,
        qel: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({this:?}, {qel:?})");

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({this:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let width: i32 = jvm.invoke_virtual(&midp_display, "getWidth", "()I", ()).await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({this:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let height: i32 = jvm.invoke_virtual(&midp_display, "getHeight", "()I", ()).await?;

        Ok(height)
    }

    async fn call_serially(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r: ClassInstanceRef<Runnable>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({this:?}, {r:?})");

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let _: () = jvm.invoke_virtual(&midp_display, "callSerially", "(Ljava/lang/Runnable;)V", (r,)).await?;

        Ok(())
    }

    async fn call_serially_with_timeout(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        runnable: ClassInstanceRef<Runnable>,
        timeout: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::callSerially({this:?}, {runnable:?}, {timeout})");

        Ok(())
    }

    async fn is_color(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::isColor({this:?})");

        Ok(false)
    }

    async fn num_colors(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::numColors({this:?})");

        Ok(0)
    }

    async fn has_pointer_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasPointerEvents({this:?})");

        Ok(false)
    }

    async fn has_pointer_motion_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasPointerMotionEvents({this:?})");

        Ok(false)
    }

    async fn has_repeat_events(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::hasRepeatEvents({this:?})");

        Ok(false)
    }

    async fn get_key_name(_: &Jvm, _: &mut WieJvmContext, key: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getKeyName({key})");

        Ok(None.into())
    }

    async fn get_bits_per_pixel(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getBitsPerPixel({this:?})");

        Ok(0)
    }

    async fn flush(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::flush({this:?})");

        Ok(())
    }

    async fn remove_jlet_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::removeJletEventListener({this:?}, {listener:?})");

        Ok(())
    }

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

    async fn ungrab_key(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::ungrabKey({this:?}, {key})");

        Ok(())
    }

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
        jvm.get_field(this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

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
