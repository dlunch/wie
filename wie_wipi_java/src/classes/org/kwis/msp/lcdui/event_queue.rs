use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::net::wie::EventQueue as WieEventQueue;

use crate::classes::org::kwis::msp::lcdui::Jlet;

// class org.kwis.msp.lcdui.EventQueue
pub struct EventQueue;

impl EventQueue {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/EventQueue",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", Self::init, Default::default()),
                JavaMethodProto::new("getNextEvent", "([I)V", Self::get_next_event, Default::default()),
                JavaMethodProto::new("dispatchEvent", "([I)V", Self::dispatch_event, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("wieEventQueue", "Lnet/wie/EventQueue;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<EventQueue>, jlet: ClassInstanceRef<Jlet>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::<init>({:?}, {:?})", &this, &jlet);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let wie_event_queue: ClassInstanceRef<WieEventQueue> = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        jvm.put_field(&mut this, "wieEventQueue", "Lnet/wie/EventQueue;", wie_event_queue).await?;

        Ok(())
    }

    async fn get_next_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::getNextEvent({:?}, {:?})", &this, &event);

        let wie_event_queue = jvm.get_field(&this, "wieEventQueue", "Lnet/wie/EventQueue;").await?;
        let _: () = jvm.invoke_virtual(&wie_event_queue, "getNextEvent", "([I)V", (event,)).await?;

        Ok(())
    }

    async fn dispatch_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::dispatchEvent({:?}, {:?})", &this, &event);

        let wie_event_queue = jvm.get_field(&this, "wieEventQueue", "Lnet/wie/EventQueue;").await?;
        let _: () = jvm.invoke_virtual(&wie_event_queue, "dispatchEvent", "([I)V", (event,)).await?;

        Ok(())
    }
}
