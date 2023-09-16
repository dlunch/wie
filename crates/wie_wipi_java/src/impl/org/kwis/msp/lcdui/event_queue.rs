use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::Jlet,
};

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

    async fn get_next_event(_: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.EventQueue::getNextEvent({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn dispatch_event(_: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.EventQueue::dispatchEvent({:#x})", this.ptr_instance);

        Ok(())
    }
}
