use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.nokia.mid.ui.DeviceControl
pub struct DeviceControl;

impl DeviceControl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/nokia/mid/ui/DeviceControl",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "setLights",
                    "(II)V",
                    Self::set_lights,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "flashLights",
                    "(J)V",
                    Self::flash_lights,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "startVibra",
                    "(IJ)V",
                    Self::start_vibra,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stopVibra",
                    "()V",
                    Self::stop_vibra,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn set_lights(_: &Jvm, _: &mut WieJvmContext, num: i32, level: i32) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::setLights({num}, {level})");

        Ok(())
    }

    async fn flash_lights(_: &Jvm, _: &mut WieJvmContext, duration: i64) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::flashLights({duration})");

        Ok(())
    }

    async fn start_vibra(_: &Jvm, _: &mut WieJvmContext, frequency: i32, duration: i64) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::startVibra({frequency}, {duration})");

        Ok(())
    }

    async fn stop_vibra(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DeviceControl::stopVibra()");

        Ok(())
    }
}
