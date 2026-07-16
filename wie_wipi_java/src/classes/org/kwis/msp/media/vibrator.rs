use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.media.Vibrator
pub struct Vibrator;

impl Vibrator {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Vibrator",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("on", "(II)V", Self::on, MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC),
                JavaMethodProto::new("off", "()V", Self::off, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn on(_: &Jvm, context: &mut WieJvmContext, level: i32, duration: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Vibrator::on({level}, {duration})");

        let duration_ms = duration.max(0) as u64;
        let intensity = (level.clamp(0, 10) * 10) as u8;
        context.system().platform().vibrate(duration_ms, intensity);

        Ok(())
    }

    async fn off(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Vibrator::off()");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_off_is_noop() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let _: () = jvm.invoke_static("org/kwis/msp/media/Vibrator", "off", "()V", ()).await?;

            Ok(())
        })
    }
}
