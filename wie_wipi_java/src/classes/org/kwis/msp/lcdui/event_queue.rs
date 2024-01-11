use alloc::{vec, vec::Vec};

use java_runtime_base::{Array, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use wie_common::KeyCode;

use crate::{
    classes::org::kwis::msp::lcdui::{Card, Display, Image, Jlet},
    WIPIJavaClassProto, WIPIJavaContxt,
};

#[repr(i32)]
enum EventQueueEvent {
    KeyEvent = 1,
    RepaintEvent = 41,
}

impl EventQueueEvent {
    fn from_raw(raw: i32) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

#[repr(i32)]
#[derive(Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
enum KeyboardEventType {
    KeyPressed = 1,
    KeyReleased = 2,
    KeyRepeated = 3,
    KeyTyped = 4,
}

impl KeyboardEventType {
    fn from_raw(raw: i32) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

#[repr(i32)]
#[allow(clippy::upper_case_acronyms)]
enum WIPIKeyCode {
    UP = -1,
    DOWN = -2,
    LEFT = -3,
    RIGHT = -4,
    FIRE = -5, // Ok

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
    fn from_key_code(keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::UP => Self::UP,
            KeyCode::DOWN => Self::DOWN,
            KeyCode::LEFT => Self::LEFT,
            KeyCode::RIGHT => Self::RIGHT,
            KeyCode::OK => Self::FIRE,
            KeyCode::NUM0 => Self::NUM0,
            KeyCode::NUM1 => Self::NUM1,
            KeyCode::NUM2 => Self::NUM2,
            KeyCode::NUM3 => Self::NUM3,
            KeyCode::NUM4 => Self::NUM4,
            KeyCode::NUM5 => Self::NUM5,
            KeyCode::NUM6 => Self::NUM6,
            KeyCode::NUM7 => Self::NUM7,
            KeyCode::NUM8 => Self::NUM8,
            KeyCode::NUM9 => Self::NUM9,
            KeyCode::HASH => Self::HASH,
            KeyCode::STAR => Self::STAR,
        }
    }
}

// class org.kwis.msp.lcdui.EventQueue
pub struct EventQueue {}

impl EventQueue {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getNextEvent", "([I)V", Self::get_next_event, JavaMethodFlag::NONE),
                JavaMethodProto::new("dispatchEvent", "([I)V", Self::dispatch_event, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(
        _: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<EventQueue>,
        jlet: JvmClassInstanceHandle<Jlet>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::<init>({:?}, {:?})", &this, &jlet);

        Ok(())
    }

    async fn get_next_event(
        jvm: &mut Jvm,
        context: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<Self>,
        mut event: JvmClassInstanceHandle<Array<i32>>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::getNextEvent({:?}, {:?})", &this, &event);

        loop {
            let maybe_event = context.system().event_queue().pop();

            if let Some(x) = maybe_event {
                let event_data = match x {
                    wie_common::Event::Redraw => vec![EventQueueEvent::RepaintEvent as _, 0, 0, 0],
                    wie_common::Event::Keydown(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyPressed as _,
                        WIPIKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    wie_common::Event::Keyup(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyReleased as _,
                        WIPIKeyCode::from_key_code(x) as _,
                        0,
                    ],
                };

                jvm.store_array(&mut event, 0, event_data)?;

                break;
            } else {
                let until = context.system().platform().now() + 16;
                context.system().sleep(until).await; // TODO we need to wait for events
            }
        }

        Ok(())
    }

    async fn dispatch_event(
        jvm: &mut Jvm,
        context: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<Self>,
        event: JvmClassInstanceHandle<Array<i32>>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::dispatchEvent({:?}, {:?})", &this, &event);

        let event = jvm.load_array(&event, 0, 4)?;

        match EventQueueEvent::from_raw(event[0]) {
            EventQueueEvent::RepaintEvent => {
                Self::repaint(jvm, context).await?;
            }
            EventQueueEvent::KeyEvent => {
                let event_type = KeyboardEventType::from_raw(event[1]);
                let code = event[2];

                tracing::debug!("KeyEvent {:?} {}", event_type, code);
                Self::key_event(jvm, event_type, code).await?;
            }
        }

        Ok(())
    }

    async fn key_event(jvm: &mut Jvm, event_type: KeyboardEventType, code: i32) -> JavaResult<()> {
        let display = Self::get_current_display(jvm).await?;
        if display.is_null() {
            return Ok(());
        }

        let card = Self::get_top_card(jvm, &display)?;
        if card.is_null() {
            return Ok(());
        }

        let _: bool = jvm
            .invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "keyNotify", "(II)Z", (event_type as i32, code))
            .await?;

        Ok(())
    }

    async fn repaint(jvm: &mut Jvm, context: &mut WIPIJavaContxt) -> JavaResult<()> {
        let display = Self::get_current_display(jvm).await?;
        if display.is_null() {
            return Ok(());
        }

        let card = Self::get_top_card(jvm, &display)?;
        if card.is_null() {
            return Ok(());
        }

        let mut graphics = jvm
            .new_class("org/kwis/msp/lcdui/Graphics", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
            .await?;

        jvm.invoke_virtual(
            &card,
            "org/kwis/msp/lcdui/Card",
            "paint",
            "(Lorg/kwis/msp/lcdui/Graphics;)V",
            [graphics.clone().into()],
        )
        .await?;

        let java_image: JvmClassInstanceHandle<Image> = jvm.get_field(&graphics, "img", "Lorg/kwis/msp/lcdui/Image;")?;

        if !java_image.is_null() {
            let image = Image::image(jvm, &java_image)?;

            // TODO temporary until we have correct gc
            let image_data = jvm.get_field(&java_image, "imgData", "[B")?;
            jvm.destroy(image_data)?;
            jvm.destroy(java_image.into())?;
            jvm.put_field(&mut graphics, "img", "Lorg/kwis/msp/lcdui/Image;", None)?;

            let mut platform = context.system().platform();
            let screen = platform.screen();

            screen.paint(&*image);
        }

        Ok(())
    }

    async fn get_current_display(jvm: &mut Jvm) -> JavaResult<JvmClassInstanceHandle<Display>> {
        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        jvm.get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;")
    }

    fn get_top_card(jvm: &mut Jvm, display: &JvmClassInstanceHandle<Display>) -> JavaResult<JvmClassInstanceHandle<Card>> {
        let cards = jvm.get_field(display, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        let card_size: i32 = jvm.get_field(display, "szCard", "I")?;

        if card_size > 0 {
            let card_data: Vec<JvmClassInstanceHandle<Card>> = jvm.load_array(&cards, 0, card_size as _)?;
            Ok(card_data[card_size as usize - 1].clone().into())
        } else {
            Ok(None.into())
        }
    }
}
