use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use super::SYSTheme;

// class wec.OEMDevice
#[java_class(name = "wec/OEMDevice", public, final)]
pub struct OEMDevice;

#[java_methods]
impl OEMDevice {
    #[method(name = "getSYSTheme", public, static)]
    async fn get_sys_theme(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<SYSTheme>> {
        tracing::warn!("stub wec.OEMDevice::getSYSTheme()");
        Ok(ClassInstanceRef::new(None))
    }
}
