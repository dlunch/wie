use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use spin::Mutex;

use wie_core_arm::{ArmCore, JumpTo, RegisteredFunction, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::SVC_CATEGORY_JAVA;

pub mod classes;
mod exception;
pub mod interface;
mod jvm_support;
mod system;

pub(crate) use exception::JavaExceptionState;
pub use interface::get_java_interface_method;
pub use jvm_support::LgtJvmSupport;
pub use system::register_java_system_svc_handler;

pub type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

async fn handle_java_svc(core: &mut ArmCore, (functions, exception_state): &mut (JavaSvcFunctions, JavaExceptionState), id: SvcId) -> Result<JumpTo> {
    let (_, lr) = core.read_pc_lr()?;
    let function = functions
        .lock()
        .get(&id.0)
        .cloned()
        .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown LGT Java SVC id {:#x}", id.0)))?;

    match function.call(core).await {
        Ok(()) => Ok(JumpTo(lr)),
        Err(WieError::JavaException(ptr_exception)) => match exception_state.unwind(core, ptr_exception)? {
            Some(resume_address) => Ok(JumpTo(resume_address)),
            None => Err(WieError::JavaException(ptr_exception)),
        },
        Err(error) => Err(error),
    }
}

pub fn register_java_svc_handler(core: &mut ArmCore, functions: &JavaSvcFunctions, exception_state: JavaExceptionState) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA, handle_java_svc, &(functions.clone(), exception_state))
}
