use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.Device
pub struct Device {}

impl Device {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Device",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("isBacklightEnabled", "()Z", Self::is_backlight_enabled, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setBacklightEnabled", "(Z)V", Self::set_backlight_enabled, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn init(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Device::<init>({:?})", &this);

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
}
