use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use spin::Mutex;

use wie_core_arm::{ArmCore, RegisteredFunction, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::SVC_CATEGORY_WIPIC;

mod context;
pub mod interface;
mod method_table;

pub(crate) type WIPICSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

async fn handle_wipic_svc(core: &mut ArmCore, svc_functions: &mut WIPICSvcFunctions, id: SvcId) -> Result<()> {
    let function = {
        let svc_functions = svc_functions.lock();
        svc_functions
            .get(&id.0)
            .cloned()
            .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown KTF WIPIC SVC id {:#x}", id.0)))?
    };

    function.call(core).await
}

pub(crate) fn register_wipic_svc_handler(core: &mut ArmCore, svc_functions: &WIPICSvcFunctions) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_WIPIC, handle_wipic_svc, svc_functions)
}
