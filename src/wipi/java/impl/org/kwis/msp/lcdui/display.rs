use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaObjectProxy, JavaResult, Jvm};

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

    fn init(_: &mut dyn Jvm) -> JavaResult<()> {
        log::debug!("Display::<init>");

        Ok(())
    }

    fn get_display(jvm: &mut dyn Jvm, _: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDisplay");

        let instance = jvm.instantiate("org/kwis/msp/lcdui/Display")?;
        jvm.call_method(&instance, "<init>", "()V", &[])?;

        Ok(instance)
    }

    fn get_default_display(jvm: &mut dyn Jvm) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDefaultDisplay");

        let instance = jvm.instantiate("org/kwis/msp/lcdui/Display")?;
        jvm.call_method(&instance, "<init>", "()V", &[])?;

        Ok(instance)
    }

    fn get_docked_card(_: &mut dyn Jvm) -> JavaResult<JavaObjectProxy> {
        log::debug!("Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    fn push_card(_: &mut dyn Jvm, _: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Display::pushCard");

        Ok(())
    }
}
