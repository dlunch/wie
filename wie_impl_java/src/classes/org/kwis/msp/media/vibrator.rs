use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{WieClassProto, WieContext};

// class org.kwis.msp.media.Vibrator
pub struct Vibrator {}

impl Vibrator {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("on", "(II)V", Self::on, JavaMethodFlag::NATIVE)],
            fields: vec![],
        }
    }

    async fn on(_: &mut Jvm, _: &mut WieContext, level: i32, duration: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Vibrator::on({}, {})", level, duration);

        Ok(())
    }
}
