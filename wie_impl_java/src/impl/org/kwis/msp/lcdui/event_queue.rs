use alloc::vec;

use wie_base::KeyCode;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::{Image as JavaImage, Jlet},
    Array,
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
enum WIPIKeyCode {
    UP = -1,
    DOWN = -4,
    LEFT = -2,
    RIGHT = -5,
    FIRE = -8, // Ok

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
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
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

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, jlet: JavaObjectProxy<Jlet>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::<init>({:#x}, {:#x})",
            this.ptr_instance,
            jlet.ptr_instance
        );

        Ok(())
    }

    async fn get_next_event(context: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, event: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::getNextEvent({:#x}, {:#x})",
            this.ptr_instance,
            event.ptr_instance
        );

        loop {
            let maybe_event = context.backend().pop_event();

            if let Some(x) = maybe_event {
                let event_data = match x {
                    wie_base::Event::Redraw => vec![EventQueueEvent::RepaintEvent as _, 0, 0, 0],
                    wie_base::Event::Keydown(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyPressed as _,
                        WIPIKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    wie_base::Event::Keyup(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyReleased as _,
                        WIPIKeyCode::from_key_code(x) as _,
                        0,
                    ],
                };

                context.store_array_i32(&event, 0, &event_data)?;

                break;
            } else {
                context.sleep(16).await; // TODO we need to wait for events
            }
        }

        Ok(())
    }

    async fn dispatch_event(context: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, event: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::dispatchEvent({:#x}, {:#x})",
            this.ptr_instance,
            event.ptr_instance
        );

        let event = context.load_array_i32(&event, 0, 4)?;

        match EventQueueEvent::from_raw(event[0]) {
            EventQueueEvent::RepaintEvent => {
                Self::repaint(context).await?;
            }
            EventQueueEvent::KeyEvent => {
                let event_type = KeyboardEventType::from_raw(event[1]);
                let code = event[2];

                tracing::debug!("KeyEvent {:?} {}", event_type, code);
                Self::key_event(context, event_type, code).await?;
            }
        }

        Ok(())
    }

    async fn key_event(context: &mut dyn JavaContext, event_type: KeyboardEventType, code: i32) -> JavaResult<()> {
        let jlet = JavaObjectProxy::new(
            context
                .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
                .await?,
        );

        let field_id = context.get_field_id("org/kwis/msp/lcdui/Jlet", "dis", "Lorg/kwis/msp/lcdui/Display;")?;
        let display = JavaObjectProxy::new(context.get_field_by_id(&jlet, field_id)?);
        if display.ptr_instance == 0 {
            return Ok(());
        }

        let cards = JavaObjectProxy::new(context.get_field(&display, "cards")?);
        let card = JavaObjectProxy::new(context.load_array_i32(&cards, 0, 1)?[0] as _);
        if card.ptr_instance == 0 {
            return Ok(());
        }

        context.call_method(&card, "keyNotify", "(II)Z", &[event_type as _, code as _]).await?;

        Ok(())
    }

    async fn repaint(context: &mut dyn JavaContext) -> JavaResult<()> {
        let jlet = JavaObjectProxy::new(
            context
                .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
                .await?,
        );

        let field_id = context.get_field_id("org/kwis/msp/lcdui/Jlet", "dis", "Lorg/kwis/msp/lcdui/Display;")?;
        let display = JavaObjectProxy::new(context.get_field_by_id(&jlet, field_id)?);
        if display.ptr_instance == 0 {
            return Ok(());
        }

        let cards = JavaObjectProxy::new(context.get_field(&display, "cards")?);
        let card = JavaObjectProxy::new(context.load_array_i32(&cards, 0, 1)?[0] as _);
        if card.ptr_instance == 0 {
            return Ok(());
        }

        let graphics = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;").await?;
        context
            .call_method(&graphics, "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", &[display.ptr_instance])
            .await?;

        context
            .call_method(&card, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", &[graphics.ptr_instance])
            .await?;

        let java_image = JavaObjectProxy::new(context.get_field(&graphics, "img")?);

        if java_image.ptr_instance != 0 {
            let image = JavaImage::image(context, &java_image)?;

            // TODO temporary until we have correct gc
            let image_data = JavaObjectProxy::new(context.get_field(&java_image.cast(), "imgData")?);
            context.destroy(image_data)?;

            let mut canvas = context.backend().screen_canvas();
            let (width, height) = (canvas.width(), canvas.height());

            canvas.draw(0, 0, width, height, &*image, 0, 0);
            drop(canvas);

            context.backend().repaint()?;
        }

        Ok(())
    }
}
