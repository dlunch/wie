use alloc::{boxed::Box, collections::BTreeMap, format, string::String, sync::Arc};

use spin::Mutex;
use wipi_types::lgt::java::{
    LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassInstance as RawJavaClassInstance,
};

use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, JumpTo, RegisteredFunction, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes};

use crate::runtime::{SVC_CATEGORY_JAVA, SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY};

mod abi;
pub mod classes;
mod exception;
mod interface;
mod jvm_support;

pub use interface::{get_java_interface_method, register_java_system_svc_handler};
pub use jvm_support::LgtJvmSupport;

pub type JavaSvcFunctions = Arc<Mutex<BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>>>;

struct JavaSvcHandler(JavaSvcFunctions);

#[async_trait::async_trait]
impl EmulatedFunction<(), JumpTo, ()> for JavaSvcHandler {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<JumpTo> {
        let id = SvcId::get(core, 0);
        let (_, lr) = core.read_pc_lr()?;
        let function = self
            .0
            .lock()
            .get(&id.0)
            .cloned()
            .ok_or_else(|| WieError::FatalError(alloc::format!("Unknown LGT Java SVC id {:#x}", id.0)))?;

        match function.call(core).await {
            Ok(()) => Ok(JumpTo(lr)),
            Err(WieError::JavaException(ptr_exception)) => match exception::unwind(core, ptr_exception)? {
                Some(resume_address) => Ok(JumpTo(resume_address)),
                None => Err(WieError::JavaException(ptr_exception)),
            },
            Err(error) => Err(error),
        }
    }
}

impl Drop for JavaSvcHandler {
    fn drop(&mut self) {
        // Method proxies retain the JVM, which also owns this table.
        let functions = core::mem::take(&mut *self.0.lock());
        drop(functions);
    }
}

async fn handle_missing_java_vtable_entry(core: &mut ArmCore, _: &mut (), id: SvcId) -> Result<JumpTo> {
    let ptr_instance = core.read_param(0)?;
    let instance: RawJavaClassInstance = read_generic(core, ptr_instance)?;
    let ptr_class: u32 = read_generic(core, instance.ptr_dispatch_table)?;
    let class: RawJavaClass = read_generic(core, ptr_class)?;
    let descriptor: RawJavaClassDescriptor = read_generic(core, class.ptr_descriptor)?;
    let class_name = String::from_utf8(read_null_terminated_string_bytes(core, descriptor.ptr_name)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT class name: {error}")))?;

    Err(WieError::Unimplemented(format!("{class_name} vtable index {}", id.0)))
}

pub fn register_java_svc_handler(core: &mut ArmCore, functions: &JavaSvcFunctions) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA, JavaSvcHandler(functions.clone()), &())?;
    core.register_svc_handler(SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY, handle_missing_java_vtable_entry, &())
}
