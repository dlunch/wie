use alloc::{string::ToString, vec, vec::Vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::Runnable;
use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult};

use wie_backend::{Event, KeyCode};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::midlet::MIDlet;

#[repr(i32)]
#[allow(clippy::enum_variant_names)]
enum EventQueueEvent {
    // TODO it's wipi event codes
    KeyEvent = 1,
    RepaintEvent = 41,
    NotifyEvent = 1000,
}

impl EventQueueEvent {
    fn from_raw(raw: i32) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

#[repr(i32)]
#[derive(Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum KeyboardEventType {
    KeyPressed = 1,
    KeyReleased = 2,
    KeyRepeated = 3,
    KeyTyped = 4,
}

impl KeyboardEventType {
    pub fn from_raw(raw: i32) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

#[repr(i32)]
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum MIDPKeyCode {
    // keycode is for skvm
    UP = 141, // MIDP Canvas's name
    DOWN = 146,
    LEFT = 142,
    RIGHT = 145,
    FIRE = 148,
    LEFT_SOFT_KEY = 6,
    RIGHT_SOFT_KEY = 7,
    CLEAR = 8,
    CALL = 10,
    HANGUP = -1,
    VOLUME_UP = 13,
    VOLUME_DOWN = 14,

    KEY_NUM0 = 48,
    KEY_NUM1 = 49,
    KEY_NUM2 = 50,
    KEY_NUM3 = 51,
    KEY_NUM4 = 52,
    KEY_NUM5 = 53,
    KEY_NUM6 = 54,
    KEY_NUM7 = 55,
    KEY_NUM8 = 56,
    KEY_NUM9 = 57,
    KEY_POUND = 35, // #
    KEY_STAR = 42,  // *
}

impl MIDPKeyCode {
    pub fn from_raw(raw: i32) -> Self {
        unsafe { core::mem::transmute(raw) }
    }

    fn from_key_code(keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::UP => Self::UP,
            KeyCode::DOWN => Self::DOWN,
            KeyCode::LEFT => Self::LEFT,
            KeyCode::RIGHT => Self::RIGHT,
            KeyCode::OK => Self::FIRE,
            KeyCode::LEFT_SOFT_KEY => Self::LEFT_SOFT_KEY,
            KeyCode::RIGHT_SOFT_KEY => Self::RIGHT_SOFT_KEY,
            KeyCode::CLEAR => Self::CLEAR,
            KeyCode::CALL => Self::CALL,
            KeyCode::HANGUP => Self::HANGUP,
            KeyCode::VOLUME_UP => Self::VOLUME_UP,
            KeyCode::VOLUME_DOWN => Self::VOLUME_DOWN,
            KeyCode::NUM0 => Self::KEY_NUM0,
            KeyCode::NUM1 => Self::KEY_NUM1,
            KeyCode::NUM2 => Self::KEY_NUM2,
            KeyCode::NUM3 => Self::KEY_NUM3,
            KeyCode::NUM4 => Self::KEY_NUM4,
            KeyCode::NUM5 => Self::KEY_NUM5,
            KeyCode::NUM6 => Self::KEY_NUM6,
            KeyCode::NUM7 => Self::KEY_NUM7,
            KeyCode::NUM8 => Self::KEY_NUM8,
            KeyCode::NUM9 => Self::KEY_NUM9,
            KeyCode::HASH => Self::KEY_POUND,
            KeyCode::STAR => Self::KEY_STAR,
        }
    }
}

// class net.wie.EventQueue
pub struct EventQueue;

impl EventQueue {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/EventQueue",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getNextEvent", "([I)V", Self::get_next_event, Default::default()),
                JavaMethodProto::new("dispatchEvent", "([I)V", Self::dispatch_event, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lnet/wie/EventQueue;",
                    Self::get_event_queue,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("eventQueue", "Lnet/wie/EventQueue;", FieldAccessFlags::STATIC),
                JavaFieldProto::new("callSeriallyEvents", "Ljava/util/Vector;", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let call_serially_events = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "callSeriallyEvents", "Ljava/util/Vector;", call_serially_events)
            .await?;

        Ok(())
    }

    // TODO this resembles WIPI's architecture for now, but we need to change it to event listener
    async fn get_next_event(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::getNextEvent({:?}, {:?})", &this, &event);

        let mut pending_timer_events = Vec::new();
        loop {
            let now = context.system().platform().now();
            let maybe_event = context.system().event_queue().pop();

            if let Some(x) = maybe_event {
                let event_data = match x {
                    Event::Redraw => vec![EventQueueEvent::RepaintEvent as _, 0, 0, 0],
                    Event::Keydown(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyPressed as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Keyup(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyReleased as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Keyrepeat(x) => vec![
                        EventQueueEvent::KeyEvent as _,
                        KeyboardEventType::KeyRepeated as _,
                        MIDPKeyCode::from_key_code(x) as _,
                        0,
                    ],
                    Event::Timer { due, callback } => {
                        // TODO we should wait for timer more efficiently
                        if due < now {
                            callback().await.map_err(|x| JavaError::FatalError(x.to_string()))?
                        } else {
                            // push it to event queue again
                            pending_timer_events.push(Event::Timer { due, callback });
                        }

                        continue;
                    }
                    // wipi notifyEvent
                    Event::Notify { r#type, param1, param2 } => vec![EventQueueEvent::NotifyEvent as i32, r#type, param1, param2],
                };

                jvm.store_array(&mut event, 0, event_data).await?;

                break;
            } else {
                let call_serially_events = jvm.get_field(&this, "callSeriallyEvents", "Ljava/util/Vector;").await?;
                if !jvm.invoke_virtual(&call_serially_events, "isEmpty", "()Z", ()).await? {
                    let event: ClassInstanceRef<Runnable> =
                        jvm.invoke_virtual(&call_serially_events, "remove", "(I)Ljava/lang/Object;", (0,)).await?;
                    let _: () = jvm.invoke_virtual(&event, "run", "()V", ()).await?;
                }

                context.system().sleep(16).await; // TODO we need to wait for events

                for event in pending_timer_events.drain(..) {
                    context.system().event_queue().push(event);
                }
            }
        }

        for event in pending_timer_events {
            context.system().event_queue().push(event);
        }

        Ok(())
    }

    async fn dispatch_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::dispatchEvent({:?}, {:?})", &this, &event);

        let current_midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;

        let display = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getDisplay",
                "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                (current_midlet,),
            )
            .await?;

        let event = jvm.load_array(&event, 0, 4).await?;
        match EventQueueEvent::from_raw(event[0]) {
            EventQueueEvent::RepaintEvent => {
                let _: () = jvm.invoke_virtual(&display, "handlePaintEvent", "()V", ()).await?;
            }
            EventQueueEvent::KeyEvent => {
                let event_type = KeyboardEventType::from_raw(event[1]);
                let code = event[2];

                let _: () = jvm.invoke_virtual(&display, "handleKeyEvent", "(II)V", (event_type as i32, code)).await?;
            }
            EventQueueEvent::NotifyEvent => {
                let r#type = event[1];
                let param1 = event[2];
                let param2 = event[3];

                let _: () = jvm
                    .invoke_virtual(&display, "handleNotifyEvent", "(III)V", (r#type, param1, param2))
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_event_queue(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("net.wie.EventQueue::getEventQueue()");

        let event_queue: ClassInstanceRef<Self> = jvm.get_static_field("net/wie/EventQueue", "eventQueue", "Lnet/wie/EventQueue;").await?;
        let event_queue = if event_queue.is_null() {
            let instance = jvm.new_class("net/wie/EventQueue", "()V", ()).await?;
            jvm.put_static_field("net/wie/EventQueue", "eventQueue", "Lnet/wie/EventQueue;", instance.clone())
                .await?;

            instance.into()
        } else {
            event_queue
        };

        Ok(event_queue)
    }

    async fn call_serially(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Runnable>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.EventQueue::callSerially({:?}, {:?})", &this, &event);

        let call_serially_events = jvm.get_field(&this, "callSeriallyEvents", "Ljava/util/Vector;").await?;
        jvm.invoke_virtual(&call_serially_events, "addElement", "(Ljava/lang/Object;)V", [event.into()])
            .await
    }
}
