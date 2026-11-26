use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::net::wie::EventQueue as WieEventQueue;

use crate::classes::org::kwis::msp::lcdui::Jlet;

// class org.kwis.msp.lcdui.EventQueue
#[java_class(name = "org/kwis/msp/lcdui/EventQueue", public)]
pub struct EventQueue {
    #[field(private)]
    wieEventQueue: ClassInstanceRef<WieEventQueue>,
}

#[java_methods]
impl EventQueue {
    #[method(constructor)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<EventQueue>, jlet: ClassInstanceRef<Jlet>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::<init>({this:?}, {jlet:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let wie_event_queue: ClassInstanceRef<WieEventQueue> = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lcdui/EventQueue",
            "wieEventQueue",
            "Lnet/wie/EventQueue;",
            wie_event_queue,
        )
        .await?;

        Ok(())
    }

    #[method(public)]
    async fn get_next_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::getNextEvent({this:?}, {event:?})");

        let wie_event_queue: ClassInstanceRef<WieEventQueue> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/EventQueue", "wieEventQueue", "Lnet/wie/EventQueue;")
            .await?;
        let _: () = jvm
            .invoke_virtual(&wie_event_queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event,))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn dispatch_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.EventQueue::dispatchEvent({this:?}, {event:?})");

        let wie_event_queue: ClassInstanceRef<WieEventQueue> = jvm
            .get_field(&this, "org/kwis/msp/lcdui/EventQueue", "wieEventQueue", "Lnet/wie/EventQueue;")
            .await?;
        let _: () = jvm
            .invoke_virtual(&wie_event_queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event,))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn post_event(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event: ClassInstanceRef<Array<i32>>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.EventQueue::postEvent({this:?}, {event:?})");

        Ok(false)
    }

    #[method(name = "postEvent", public, static)]
    async fn post_event_static(_: &Jvm, _: &mut WieJvmContext, id: i32, event: ClassInstanceRef<Array<i32>>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.EventQueue::postEvent({id}, {event:?})");

        Ok(())
    }
}
