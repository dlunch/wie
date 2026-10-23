mod clet_wrapper;
mod clet_wrapper_card;
mod lgt_class_loader;

use alloc::string::ToString;

use jvm::{Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;

use crate::runtime::with_arm_context;

#[derive(Clone)]
pub struct CletWrapperContext {
    pub core: ArmCore,
}

impl CletWrapperContext {
    pub async fn call_function(&mut self, jvm: &Jvm, address: u32, args: &[u32]) -> JvmResult<()> {
        match self.core.run_function(address, args).await {
            Ok(()) => Ok(()),
            Err(error) => {
                let message = with_arm_context(&self.core, error).to_string();
                Err(jvm.exception("net/wie/WieError", &message).await)
            }
        }
    }
}

pub use self::{clet_wrapper::CletWrapper, clet_wrapper_card::CletWrapperCard, lgt_class_loader::LgtClassLoader};
