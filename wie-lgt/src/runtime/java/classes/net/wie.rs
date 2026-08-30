mod clet_wrapper;
mod clet_wrapper_card;
mod lgt_class_loader;

use wie_core_arm::ArmCore;

#[derive(Clone)]
pub struct CletWrapperContext {
    pub core: ArmCore,
}

pub use self::{clet_wrapper::CletWrapper, clet_wrapper_card::CletWrapperCard, lgt_class_loader::LgtClassLoader};
