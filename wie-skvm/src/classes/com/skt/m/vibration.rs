use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
// class com.skt.m.Vibration
#[java_class(name = "com/skt/m/Vibration", public, final)]
pub struct Vibration;

#[java_methods]
impl Vibration {
    #[method(public, static)]
    async fn get_level_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.Vibration::getLevelNum()");

        Ok(10)
    }

    #[method(public, static)]
    async fn start(_jvm: &Jvm, context: &mut WieJvmContext, level: i32, timeout: i32) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Vibration::start({level}, {timeout})");

        let duration_ms = timeout.max(0) as u64;
        let intensity = (level.clamp(0, 10) * 10) as u8;
        context.system().platform().vibrate(duration_ms, intensity);

        Ok(())
    }

    #[method(public, static)]
    async fn stop(_jvm: &Jvm, context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Vibration::stop()");

        context.system().platform().vibrate(0, 0);

        Ok(())
    }

    #[method(public, static)]
    async fn is_supported(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::debug!("com.skt.m.Vibration::isSupported()");

        Ok(true)
    }
}
