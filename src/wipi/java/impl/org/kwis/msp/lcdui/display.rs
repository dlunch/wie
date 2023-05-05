use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaObjectProxy, Jvm};

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
            ],
        }
    }

    fn init(_: &mut dyn Jvm) {
        log::debug!("Display::<init>");
    }

    fn get_display(_: &mut dyn Jvm, _: JavaObjectProxy) -> JavaObjectProxy {
        log::debug!("Display::getDisplay");

        JavaObjectProxy::new(0)
    }

    fn get_default_display(_: &mut dyn Jvm) -> JavaObjectProxy {
        log::debug!("Display::getDefaultDisplay");

        JavaObjectProxy::new(0)
    }

    fn get_docked_card(_: &mut dyn Jvm) -> JavaObjectProxy {
        log::debug!("Display::getDockedCard");

        JavaObjectProxy::new(0)
    }

    fn push_card(_: &mut dyn Jvm, _: JavaObjectProxy) {
        log::debug!("Display::pushCard");
    }
}
