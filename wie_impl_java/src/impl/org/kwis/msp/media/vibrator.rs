use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    JavaContext, JavaMethodFlag, JavaResult,
};

// class org.kwis.msp.media.Vibrator
pub struct Vibrator {}

impl Vibrator {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("on", "(II)V", Self::on, JavaMethodFlag::NATIVE)],
            fields: vec![],
        }
    }

    async fn on(_: &mut dyn JavaContext, level: i32, duration: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Vibrator::on({}, {})", level, duration);

        Ok(())
    }
}
