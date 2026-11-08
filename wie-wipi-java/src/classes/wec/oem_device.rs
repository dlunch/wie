use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::SYSTheme;

// class wec.OEMDevice
pub struct OEMDevice;

impl OEMDevice {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/OEMDevice",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSYSTheme",
                "()Lwec/SYSTheme;",
                Self::get_sys_theme,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn get_sys_theme(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<SYSTheme>> {
        tracing::warn!("stub wec.OEMDevice::getSYSTheme()");
        Ok(ClassInstanceRef::new(None))
    }
}
