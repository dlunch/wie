use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaObjectProxy, JavaResult};

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

    fn init(_: &mut JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Backlight::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    fn always_on(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Backlight::alwaysOn");

        Ok(())
    }
}
