use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
// class com.skt.m.Vibration
pub struct Vibration;

impl Vibration {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Vibration",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("getLevelNum", "()I", Self::get_level_num, MethodAccessFlags::STATIC),
                JavaMethodProto::new("start", "(II)V", Self::start, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn get_level_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.Vibration::getLevelNum()");

        Ok(10)
    }

    async fn start(_jvm: &Jvm, _context: &mut WieJvmContext, level: i32, timeout: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Vibration::start({}, {})", level, timeout);

        Ok(())
    }
}
