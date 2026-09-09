use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use spin::Mutex;

use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, RegisteredFunction, SvcId};
use wie_util::{Result, WieError};

use crate::runtime::SVC_CATEGORY_JAVA;

pub mod interface;
pub mod jvm_support;

pub type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

struct JavaSvcHandler(JavaSvcFunctions);

#[async_trait::async_trait]
impl EmulatedFunction<(), (), ()> for JavaSvcHandler {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<()> {
        let id = SvcId::get(core, 0);
        let function = self
            .0
            .lock()
            .get(&id.0)
            .cloned()
            .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown KTF Java SVC id {:#x}", id.0)))?;

        function.call(core).await
    }
}

impl Drop for JavaSvcHandler {
    fn drop(&mut self) {
        // Method proxies retain the JVM, which also owns this table.
        let functions = core::mem::take(&mut *self.0.lock());
        drop(functions);
    }
}

pub fn register_java_svc_handler(core: &mut ArmCore, svc_functions: &JavaSvcFunctions) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA, JavaSvcHandler(svc_functions.clone()), &())
}
