use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaObjectProxy, JavaResult};

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
        }
    }

    fn init(_: JavaContext) -> JavaResult<()> {
        log::debug!("Display::<init>");

        Ok(())
    }

    fn get_display(mut context: JavaContext, _: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDisplay");

        let instance = context.bridge.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        context.bridge.call_method(&instance, "<init>", "()V", &[])?;

        Ok(instance)
    }

    fn get_default_display(mut context: JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDefaultDisplay");

        let instance = context.bridge.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        context.bridge.call_method(&instance, "<init>", "()V", &[])?;

        Ok(instance)
    }

    fn get_docked_card(_: JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    fn push_card(_: JavaContext, _: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::pushCard");

        Ok(())
    }

    fn add_jlet_event_listener(_: JavaContext, _: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::addJletEventListener");

        Ok(())
    }
}
