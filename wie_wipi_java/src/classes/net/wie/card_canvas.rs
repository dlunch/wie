use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::{Class, String};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::{
    javax::microedition::lcdui::{Display as MidpDisplay, Graphics as MidpGraphics},
    net::wie::MIDPKeyCode,
};

use crate::classes::org::kwis::msp::lcdui::{Card, Display};

#[repr(i32)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum WIPIKeyCode {
    UP = -1,
    DOWN = -2,
    LEFT = -3,
    RIGHT = -4,
    FIRE = -5, // Ok
    LEFT_SOFT_KEY = -6,
    RIGHT_SOFT_KEY = -7,
    CLEAR = -16,
    CALL = -10,
    HANGUP = -11,
    VOLUME_UP = -13,
    VOLUME_DOWN = -14,

    NUM0 = 48,
    NUM1 = 49,
    NUM2 = 50,
    NUM3 = 51,
    NUM4 = 52,
    NUM5 = 53,
    NUM6 = 54,
    NUM7 = 55,
    NUM8 = 56,
    NUM9 = 57,
    HASH = 35, // #
    STAR = 42, // *
}

impl WIPIKeyCode {
    pub fn from_raw(value: i32) -> Option<Self> {
        Some(match value {
            x if x == Self::UP as i32 => Self::UP,
            x if x == Self::DOWN as i32 => Self::DOWN,
            x if x == Self::LEFT as i32 => Self::LEFT,
            x if x == Self::RIGHT as i32 => Self::RIGHT,
            x if x == Self::FIRE as i32 => Self::FIRE,
            x if x == Self::LEFT_SOFT_KEY as i32 => Self::LEFT_SOFT_KEY,
            x if x == Self::RIGHT_SOFT_KEY as i32 => Self::RIGHT_SOFT_KEY,
            x if x == Self::CLEAR as i32 => Self::CLEAR,
            x if x == Self::CALL as i32 => Self::CALL,
            x if x == Self::HANGUP as i32 => Self::HANGUP,
            x if x == Self::VOLUME_UP as i32 => Self::VOLUME_UP,
            x if x == Self::VOLUME_DOWN as i32 => Self::VOLUME_DOWN,
            x if x == Self::NUM0 as i32 => Self::NUM0,
            x if x == Self::NUM1 as i32 => Self::NUM1,
            x if x == Self::NUM2 as i32 => Self::NUM2,
            x if x == Self::NUM3 as i32 => Self::NUM3,
            x if x == Self::NUM4 as i32 => Self::NUM4,
            x if x == Self::NUM5 as i32 => Self::NUM5,
            x if x == Self::NUM6 as i32 => Self::NUM6,
            x if x == Self::NUM7 as i32 => Self::NUM7,
            x if x == Self::NUM8 as i32 => Self::NUM8,
            x if x == Self::NUM9 as i32 => Self::NUM9,
            x if x == Self::HASH as i32 => Self::HASH,
            x if x == Self::STAR as i32 => Self::STAR,
            _ => return None,
        })
    }

    pub fn from_midp_raw(keycode: i32) -> i32 {
        match MIDPKeyCode::from_raw(keycode) {
            Some(MIDPKeyCode::UP) => Self::UP as i32,
            Some(MIDPKeyCode::DOWN) => Self::DOWN as i32,
            Some(MIDPKeyCode::LEFT) => Self::LEFT as i32,
            Some(MIDPKeyCode::RIGHT) => Self::RIGHT as i32,
            Some(MIDPKeyCode::FIRE) => Self::FIRE as i32,
            Some(MIDPKeyCode::LEFT_SOFT_KEY) => Self::LEFT_SOFT_KEY as i32,
            Some(MIDPKeyCode::RIGHT_SOFT_KEY) => Self::RIGHT_SOFT_KEY as i32,
            Some(MIDPKeyCode::CLEAR) => Self::CLEAR as i32,
            Some(MIDPKeyCode::CALL) => Self::CALL as i32,
            Some(MIDPKeyCode::HANGUP) => Self::HANGUP as i32,
            Some(MIDPKeyCode::VOLUME_UP) => Self::VOLUME_UP as i32,
            Some(MIDPKeyCode::VOLUME_DOWN) => Self::VOLUME_DOWN as i32,
            Some(MIDPKeyCode::KEY_NUM0) => Self::NUM0 as i32,
            Some(MIDPKeyCode::KEY_NUM1) => Self::NUM1 as i32,
            Some(MIDPKeyCode::KEY_NUM2) => Self::NUM2 as i32,
            Some(MIDPKeyCode::KEY_NUM3) => Self::NUM3 as i32,
            Some(MIDPKeyCode::KEY_NUM4) => Self::NUM4 as i32,
            Some(MIDPKeyCode::KEY_NUM5) => Self::NUM5 as i32,
            Some(MIDPKeyCode::KEY_NUM6) => Self::NUM6 as i32,
            Some(MIDPKeyCode::KEY_NUM7) => Self::NUM7 as i32,
            Some(MIDPKeyCode::KEY_NUM8) => Self::NUM8 as i32,
            Some(MIDPKeyCode::KEY_NUM9) => Self::NUM9 as i32,
            Some(MIDPKeyCode::KEY_POUND) => Self::HASH as i32,
            Some(MIDPKeyCode::KEY_STAR) => Self::STAR as i32,
            None => keycode,
        }
    }
}

// class net.wie.CardCanvas
pub struct CardCanvas;

impl CardCanvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/CardCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, Default::default()),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, Default::default()),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, Default::default()),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, Default::default()),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, Default::default()),
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, Default::default()),
                // wie private
                JavaMethodProto::new("handleNotifyEvent", "(III)V", Self::handle_notify_event, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("cards", "Ljava/util/Vector;", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;

        let _: () = jvm.invoke_virtual(&this, "setFullScreenMode", "(Z)V", (true,)).await?;

        let cards = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "cards", "Ljava/util/Vector;", cards).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<MidpGraphics>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::paint({this:?}, {g:?})");

        let graphics = jvm
            .new_class("org/kwis/msp/lcdui/Graphics", "(Ljavax/microedition/lcdui/Graphics;)V", (g,))
            .await?;

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let _: () = jvm
                .invoke_virtual(&card, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", (graphics.clone(),))
                .await?;
        }

        Ok(())
    }

    async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::keyPressed({this:?}, {key_code})");

        let key_code = WIPIKeyCode::from_midp_raw(key_code);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (1i32, key_code)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::keyRepeated({this:?}, {key_code})");

        let key_code = WIPIKeyCode::from_midp_raw(key_code);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (3i32, key_code)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::keyReleased({this:?}, {key_code})");

        let key_code = WIPIKeyCode::from_midp_raw(key_code);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (2i32, key_code)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::pushCard({this:?}, {c:?})");

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let _: () = jvm.invoke_virtual(&cards, "addElement", "(Ljava/lang/Object;)V", (c.clone(),)).await?;

        let _: () = jvm
            .invoke_virtual(&c, "setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", (this.clone(),))
            .await?;
        let _: () = jvm.invoke_virtual(&c, "showNotify", "(Z)V", (true,)).await?;

        let _: () = jvm.invoke_virtual(&this, "repaint", "()V", ()).await?;

        // HACK: disable java level paint on clet app
        let class: ClassInstanceRef<Class> = jvm.invoke_virtual(&c, "getClass", "()Ljava/lang/Class;", ()).await?;
        let class_name: ClassInstanceRef<String> = jvm.invoke_virtual(&class, "getName", "()Ljava/lang/String;", ()).await?;
        let class_name_str = JavaLangString::to_rust_string(jvm, &class_name).await?;

        if class_name_str == "CletCard" || class_name_str == "net/wie/CletWrapperCard" {
            let wipi_display: ClassInstanceRef<Display> = jvm
                .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                .await?;
            let midp_display: ClassInstanceRef<MidpDisplay> =
                jvm.get_field(&wipi_display, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
            let _: () = jvm.invoke_virtual(&midp_display, "disablePaint", "()V", ()).await?;
        }

        Ok(())
    }

    async fn remove_all_cards(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::removeAllCards");

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let _: () = jvm
                .invoke_virtual(&card, "setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", (None,))
                .await?;
            let _: () = jvm.invoke_virtual(&card, "showNotify", "(Z)V", (false,)).await?;
        }

        let _: () = jvm.invoke_virtual(&cards, "removeAllElements", "()V", ()).await?;

        Ok(())
    }

    async fn handle_notify_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::handleNotifyEvent({this:?}, {type}, {param1}, {param2})");

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let top_card = jvm.invoke_virtual(&cards, "firstElement", "()Ljava/lang/Object;", ()).await?;

        let _: () = jvm.invoke_virtual(&top_card, "notifyEvent", "(III)V", (r#type, param1, param2)).await?;

        Ok(())
    }
}
