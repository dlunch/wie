use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getDisplay", "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;", Self::get_display),
                JavaMethodProto::new("getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", Self::get_default_display),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDisplay({:#x}", a0.ptr_instance);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        // context.call_method(&instance, "<init>", "()V", &[]).await?; // TODO

        Ok(instance)
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDefaultDisplay");

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        // context.call_method(&instance, "<init>", "()V", &[]).await?; // TODO

        Ok(instance)
    }

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    async fn push_card(_: &mut dyn JavaContext, instance: JavaObjectProxy, a1: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::pushCard({:#x}, {:#x})", instance.ptr_instance, a1.ptr_instance);

        Ok(())
    }

    async fn add_jlet_event_listener(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::addJletEventListener({:#x})", a0.ptr_instance);

        Ok(())
    }
}
