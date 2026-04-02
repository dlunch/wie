use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use spin::Mutex;

use wie_core_arm::{ArmCore, RegisteredFunction, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::SVC_CATEGORY_JAVA;

pub mod interface;
pub mod jvm_support;

pub(crate) type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

pub(crate) fn register_java_svc_handler(core: &mut ArmCore, svc_functions: &JavaSvcFunctions) -> Result<()> {
    async fn handle_java_svc(core: &mut ArmCore, svc_functions: &mut JavaSvcFunctions, id: SvcId) -> Result<()> {
        let function = {
            let svc_functions = svc_functions.lock();
            svc_functions
                .get(&id.0)
                .cloned()
                .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown KTF Java SVC id {:#x}", id.0)))?
        };

        function.call(core).await
    }

    core.register_svc_handler(SVC_CATEGORY_JAVA, handle_java_svc, svc_functions)
}
