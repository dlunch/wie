use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("alwaysOn", "()V", Self::always_on),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Backlight::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn always_on(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::debug!("Backlight::alwaysOn");

        Ok(())
    }
}
