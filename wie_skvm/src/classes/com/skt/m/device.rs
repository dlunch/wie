use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.Device
pub struct Device;

impl Device {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Device",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("setColorMode", "(I)V", Self::set_color_mode, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isBacklightEnabled", "()Z", Self::is_backlight_enabled, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setBacklightEnabled", "(Z)V", Self::set_backlight_enabled, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setKeyToneEnabled", "(Z)V", Self::set_key_tone_enabled, MethodAccessFlags::STATIC),
                JavaMethodProto::new("enableRestoreLCD", "(Z)V", Self::enable_restore_lcd, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setKeyRepeatTime", "(II)V", Self::set_key_repeat_time, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn set_color_mode(_jvm: &Jvm, _context: &mut WieJvmContext, mode: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setColorMode({})", mode);

        Ok(())
    }

    async fn is_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::isBacklightEnabled()");

        Ok(true)
    }

    async fn set_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setBacklightEnabled({:?})", enabled);

        Ok(())
    }

    async fn set_key_tone_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyToneEnabled({:?})", enabled);

        Ok(())
    }

    async fn enable_restore_lcd(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::enableRestoreLCD({:?})", enabled);

        Ok(())
    }

    async fn set_key_repeat_time(_jvm: &Jvm, _context: &mut WieJvmContext, delay: i32, interval: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyRepeatTime({}, {})", delay, interval);

        Ok(())
    }
}
