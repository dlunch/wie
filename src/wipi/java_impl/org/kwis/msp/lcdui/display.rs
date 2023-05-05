use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl, Jvm};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![
                JavaMethodImpl {
                    name: "<init>".into(),
                    signature: "()V".into(),
                    body: Box::new(Self::init),
                },
                JavaMethodImpl {
                    name: "getDisplay".into(),
                    signature: "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;".into(),
                    body: Box::new(Self::get_display),
                },
                JavaMethodImpl {
                    name: "getDefaultDisplay".into(),
                    signature: "()Lorg/kwis/msp/lcdui/Display;".into(),
                    body: Box::new(Self::get_default_display),
                },
                JavaMethodImpl {
                    name: "getDockedCard".into(),
                    signature: "()Lorg/kwis/msp/lcdui/Card;".into(),
                    body: Box::new(Self::get_docked_card),
                },
                JavaMethodImpl {
                    name: "pushCard".into(),
                    signature: "(Lorg/kwis/msp/lcdui/Card;)V".into(),
                    body: Box::new(Self::push_card),
                },
            ],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Display::<init>");

        0
    }

    fn get_display(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Display::getDisplay");

        0
    }

    fn get_default_display(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Display::getDefaultDisplay");

        0
    }

    fn get_docked_card(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Display::getDockedCard");

        0
    }

    fn push_card(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Display::pushCard");

        0
    }
}
