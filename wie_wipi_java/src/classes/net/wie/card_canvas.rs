use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::{javax::microedition::lcdui::Graphics, net::wie::MIDPKeyCode};

use crate::classes::org::kwis::msp::lcdui::Card;

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
    pub fn from_raw(value: i32) -> Self {
        unsafe { core::mem::transmute(value) }
    }

    pub fn from_midp_key_code(keycode: MIDPKeyCode) -> Self {
        match keycode {
            MIDPKeyCode::UP => Self::UP,
            MIDPKeyCode::DOWN => Self::DOWN,
            MIDPKeyCode::LEFT => Self::LEFT,
            MIDPKeyCode::RIGHT => Self::RIGHT,
            MIDPKeyCode::FIRE => Self::FIRE,
            MIDPKeyCode::LEFT_SOFT_KEY => Self::LEFT_SOFT_KEY,
            MIDPKeyCode::RIGHT_SOFT_KEY => Self::RIGHT_SOFT_KEY,
            MIDPKeyCode::CLEAR => Self::CLEAR,
            MIDPKeyCode::CALL => Self::CALL,
            MIDPKeyCode::HANGUP => Self::HANGUP,
            MIDPKeyCode::VOLUME_UP => Self::VOLUME_UP,
            MIDPKeyCode::VOLUME_DOWN => Self::VOLUME_DOWN,
            MIDPKeyCode::KEY_NUM0 => Self::NUM0,
            MIDPKeyCode::KEY_NUM1 => Self::NUM1,
            MIDPKeyCode::KEY_NUM2 => Self::NUM2,
            MIDPKeyCode::KEY_NUM3 => Self::NUM3,
            MIDPKeyCode::KEY_NUM4 => Self::NUM4,
            MIDPKeyCode::KEY_NUM5 => Self::NUM5,
            MIDPKeyCode::KEY_NUM6 => Self::NUM6,
            MIDPKeyCode::KEY_NUM7 => Self::NUM7,
            MIDPKeyCode::KEY_NUM8 => Self::NUM8,
            MIDPKeyCode::KEY_NUM9 => Self::NUM9,
            MIDPKeyCode::KEY_POUND => Self::HASH,
            MIDPKeyCode::KEY_STAR => Self::STAR,
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
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::<init>({:?})", this);

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;

        let _: () = jvm.invoke_virtual(&this, "setFullScreenMode", "(Z)V", (true,)).await?;

        let cards = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "cards", "Ljava/util/Vector;", cards).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::paint({:?}, {:?})", this, g);

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
        tracing::debug!("net.wie.CardCanvas::keyPressed({:?}, {})", this, key_code);

        let key_code = WIPIKeyCode::from_midp_key_code(MIDPKeyCode::from_raw(key_code));

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (1i32, key_code as i32)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn key_repeated(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::keyRepeated({:?}, {})", this, key_code);

        let key_code = WIPIKeyCode::from_midp_key_code(MIDPKeyCode::from_raw(key_code));

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (3i32, key_code as i32)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::keyReleased({:?}, {})", this, key_code);

        let key_code = WIPIKeyCode::from_midp_key_code(MIDPKeyCode::from_raw(key_code));

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (2i32, key_code as i32)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("net.wie.CardCanvas::pushCard({:?}, {:?})", &this, &c);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let _: () = jvm.invoke_virtual(&cards, "addElement", "(Ljava/lang/Object;)V", (c.clone(),)).await?;

        let _: () = jvm
            .invoke_virtual(&c, "setCanvas", "(Ljavax/microedition/lcdui/Canvas;)V", (this.clone(),))
            .await?;
        let _: () = jvm.invoke_virtual(&c, "showNotify", "(Z)V", (true,)).await?;

        let _: () = jvm.invoke_virtual(&this, "repaint", "()V", ()).await?;

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
        tracing::debug!("net.wie.CardCanvas::handleNotifyEvent({this:?}, {}, {param1}, {param2})", r#type);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let top_card = jvm.invoke_virtual(&cards, "firstElement", "()Ljava/lang/Object;", ()).await?;

        let _: () = jvm.invoke_virtual(&top_card, "notifyEvent", "(III)V", (r#type, param1, param2)).await?;

        Ok(())
    }
}
