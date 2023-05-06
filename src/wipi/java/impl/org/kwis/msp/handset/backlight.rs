use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("alwaysOn", "()V", Self::always_on),
            ],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Backlight::<init>");

        Ok(())
    }

    fn always_on(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Backlight::alwaysOn");

        Ok(())
    }
}
