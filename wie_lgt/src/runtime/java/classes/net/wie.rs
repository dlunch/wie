mod clet_wrapper;
mod clet_wrapper_card;

use wie_core_arm::ArmCore;

#[derive(Clone)]
pub struct CletWrapperContext {
    pub core: ArmCore,
}

pub use self::{clet_wrapper::CletWrapper, clet_wrapper_card::CletWrapperCard};
