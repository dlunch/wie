use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("<init>", "(I)V", Self::init_1),
                JavaMethodProto::new("getWidth", "()I", Self::get_width),
                JavaMethodProto::new("getHeight", "()I", Self::get_height),
            ],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Card::<init>");

        Ok(())
    }

    fn init_1(_: &mut JavaContext, _: u32) -> JavaResult<()> {
        log::debug!("Card::<init>");

        Ok(())
    }

    fn get_width(_: &mut JavaContext) -> JavaResult<u32> {
        log::debug!("Card::get_width");

        Ok(320) // TODO: hardcoded
    }

    fn get_height(_: &mut JavaContext) -> JavaResult<u32> {
        log::debug!("Card::get_height");

        Ok(480) // TODO: hardcoded
    }
}
