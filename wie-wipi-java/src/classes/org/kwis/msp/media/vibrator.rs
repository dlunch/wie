use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.media.Vibrator
#[java_class(name = "org/kwis/msp/media/Vibrator", public)]
pub struct Vibrator;

#[java_methods]
impl Vibrator {
    #[method(public, native, static)]
    async fn on(_: &Jvm, context: &mut WieJvmContext, level: i32, duration: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Vibrator::on({level}, {duration})");

        let duration_ms = duration.max(0) as u64;
        let intensity = (level.clamp(0, 10) * 10) as u8;
        context.system().platform().vibrate(duration_ms, intensity);

        Ok(())
    }

    #[method(public, static)]
    async fn off(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Vibrator::off()");

        Ok(())
    }
}
