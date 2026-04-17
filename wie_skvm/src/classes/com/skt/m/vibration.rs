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
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isSupported", "()Z", Self::is_supported, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn get_level_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.Vibration::getLevelNum()");

        Ok(10)
    }

    async fn start(_jvm: &Jvm, context: &mut WieJvmContext, level: i32, timeout: i32) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Vibration::start({level}, {timeout})");

        let duration_ms = timeout.max(0) as u64;
        let intensity = (level.clamp(0, 10) * 10) as u8;
        context.system().platform().vibrate(duration_ms, intensity);

        Ok(())
    }

    async fn stop(_jvm: &Jvm, context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Vibration::stop()");

        context.system().platform().vibrate(0, 0);

        Ok(())
    }

    async fn is_supported(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::debug!("com.skt.m.Vibration::isSupported()");

        Ok(true)
    }
}
