mod init;
mod java;
mod svc_ids;
mod wipi_c;

use alloc::format;

use wie_core_arm::ArmCore;
use wie_util::WieError;

use crate::emulator::IMAGE_BASE;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_JAVA_INTERFACE: u32 = 2;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_JAVA: u32 = 4;

pub use self::java::jvm_support::{KtfJvmSupport, KtfJvmThreadContext};

pub fn with_arm_context(core: &ArmCore, error: WieError) -> WieError {
    match error {
        WieError::JavaException(_) | WieError::JavaExceptionUnwind { .. } => error,
        error => {
            let context = core.dump_reg_stack(IMAGE_BASE);
            match error {
                WieError::Unimplemented(message) => WieError::Unimplemented(format!("{message}{context}")),
                WieError::FatalError(message) => WieError::FatalError(format!("{message}{context}")),
                error => WieError::FatalError(format!("{error}{context}")),
            }
        }
    }
}
