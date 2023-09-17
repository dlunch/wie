use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    r#impl::org::kwis::msp::lcdui::EventQueue,
    JavaFieldAccessFlag, JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "getActiveJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lorg/kwis/msp/lcdui/EventQueue;",
                    Self::get_event_queue,
                    JavaMethodFlag::NONE,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("eq", "Lorg/kwis/msp/lcdui/EventQueue;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", JavaFieldAccessFlag::STATIC),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Jlet>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>");

        let display = context.instantiate("Lorg/kwis/msp/lcdui/Display;").await?;
        context
            .call_method(
                &display.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                &[this.ptr_instance, 0],
            )
            .await?;

        context.put_field(&this.cast(), "dis", display.ptr_instance)?;

        let event_queue = context.instantiate("Lorg/kwis/msp/lcdui/EventQueue;").await?;
        context
            .call_method(&event_queue.cast(), "<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", &[this.ptr_instance])
            .await?;
        context.put_field(&this.cast(), "eq", event_queue.ptr_instance)?;

        context.put_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", this.ptr_instance)?;

        Ok(())
    }

    async fn get_active_jlet(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = JavaObjectProxy::new(context.get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive")?);
        Ok(jlet)
    }

    async fn get_event_queue(context: &mut dyn JavaContext, this: JavaObjectProxy<Jlet>) -> JavaResult<JavaObjectProxy<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue");

        let eq = JavaObjectProxy::new(context.get_field(&this.cast(), "eq")?);

        Ok(eq)
    }
}
