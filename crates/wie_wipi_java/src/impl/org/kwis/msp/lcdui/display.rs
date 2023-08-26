use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::{
        java::lang::String,
        org::kwis::msp::lcdui::{Card, JletEventListener},
    },
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: "java/lang/Object",
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                    Self::get_display,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getDockedCard",
                    "()Lorg/kwis/msp/lcdui/Card;",
                    Self::get_docked_card,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                    JavaMethodAccessFlag::NONE,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("card", "Lorg/kwis/msp/lcdui/Card;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::STATIC),
            ],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Display>) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Display::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, str: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Display>> {
        log::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:#x})", str.ptr_instance);

        let display = context.get_static_field("org/kwis/msp/lcdui/Display", "display")?;
        if display == 0 {
            let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?.cast();
            context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

            context.put_static_field("org/kwis/msp/lcdui/Display", "display", instance.ptr_instance)?;

            Ok(instance)
        } else {
            Ok(JavaObjectProxy::new(display))
        }
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Display>> {
        log::warn!("stub org.kwis.msp.lcdui.Display::getDefaultDisplay");

        let ptr_instance = context
            .call_static_method(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                &[0],
            )
            .await?;

        Ok(JavaObjectProxy::new(ptr_instance))
    }

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Card>> {
        log::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    async fn push_card(context: &mut dyn JavaContext, this: JavaObjectProxy<Display>, c: JavaObjectProxy<Card>) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Display::pushCard({:#x}, {:#x})",
            this.ptr_instance,
            c.ptr_instance
        );

        let card = context.get_field(&this.cast(), "card")?;
        if card == 0 {
            context.put_field(&this.cast(), "card", c.ptr_instance)?;
        }

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<Display>,
        qel: JavaObjectProxy<JletEventListener>,
    ) -> JavaResult<()> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Display::addJletEventListener({:#x}, {:#x})",
            this.ptr_instance,
            qel.ptr_instance
        );

        Ok(())
    }
}
