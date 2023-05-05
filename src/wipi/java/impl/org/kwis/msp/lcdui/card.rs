use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaResult, Jvm};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("<init>", "(I)V", Self::init_1),
            ],
        }
    }

    fn init(_: &mut dyn Jvm) -> JavaResult<()> {
        log::debug!("Card::<init>");

        Ok(())
    }

    fn init_1(_: &mut dyn Jvm, _: u32) -> JavaResult<()> {
        log::debug!("Card::<init>");

        Ok(())
    }
}
