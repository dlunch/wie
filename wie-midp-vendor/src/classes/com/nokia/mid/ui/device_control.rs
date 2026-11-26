use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class com.nokia.mid.ui.DeviceControl
#[java_class(name = "com/nokia/mid/ui/DeviceControl", public)]
pub struct DeviceControl;

#[java_methods]
impl DeviceControl {
    #[method(public, static)]
    async fn set_lights(_: &Jvm, _: &mut WieJvmContext, num: i32, level: i32) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::setLights({num}, {level})");

        Ok(())
    }

    #[method(public, static)]
    async fn flash_lights(_: &Jvm, _: &mut WieJvmContext, duration: i64) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::flashLights({duration})");

        Ok(())
    }

    #[method(public, static)]
    async fn start_vibra(_: &Jvm, _: &mut WieJvmContext, frequency: i32, duration: i64) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::startVibra({frequency}, {duration})");

        Ok(())
    }

    #[method(public, static)]
    async fn stop_vibra(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::stopVibra()");

        Ok(())
    }
}
