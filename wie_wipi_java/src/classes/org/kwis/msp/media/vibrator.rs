use alloc::vec;

use java_class_proto::{JavaMethodProto, JavaResult};
use java_constants::MethodAccessFlags;
use jvm::Jvm;

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.media.Vibrator
pub struct Vibrator {}

impl Vibrator {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("on", "(II)V", Self::on, MethodAccessFlags::NATIVE)],
            fields: vec![],
        }
    }

    async fn on(_: &mut Jvm, _: &mut WIPIJavaContext, level: i32, duration: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Vibrator::on({}, {})", level, duration);

        Ok(())
    }
}
