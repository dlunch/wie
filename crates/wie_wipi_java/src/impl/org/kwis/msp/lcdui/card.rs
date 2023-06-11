use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

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
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Card::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn init_1(_: &mut dyn JavaContext, instance: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Card::<init>({:#x}, {})", instance.ptr_instance, a0);

        Ok(())
    }

    async fn get_width(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Card::get_width");

        Ok(320) // TODO: hardcoded
    }

    async fn get_height(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::warn!("stub org.kwis.msp.lcdui.Card::get_height");

        Ok(480) // TODO: hardcoded
    }
}
