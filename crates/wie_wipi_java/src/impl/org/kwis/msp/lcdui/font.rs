use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Font
pub struct Font {}

impl Font {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getHeight", "()I", Self::get_height),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Font::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_height(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::warn!("stub Font::get_height");

        Ok(12) // TODO: hardcoded
    }
}
