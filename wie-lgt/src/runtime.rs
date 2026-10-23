mod globals;
pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

use alloc::format;

use wie_core_arm::ArmCore;
use wie_util::WieError;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_JAVA_SYSTEM: u32 = 2;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_JAVA: u32 = 4;
const SVC_CATEGORY_STDLIB: u32 = 5;
const SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY: u32 = 6;

pub use globals::SUPPORT_CONTEXT_BASE;
pub use java::{LgtJvmSupport, exception};

pub fn with_arm_context(core: &ArmCore, error: WieError) -> WieError {
    match error {
        WieError::JavaException(_) => error,
        error => {
            let context = core.dump_reg_stack(0x1000);
            match error {
                WieError::Unimplemented(message) => WieError::Unimplemented(format!("{message}{context}")),
                WieError::FatalError(message) => WieError::FatalError(format!("{message}{context}")),
                error => WieError::FatalError(format!("{error}{context}")),
            }
        }
    }
}
