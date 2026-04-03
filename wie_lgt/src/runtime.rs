pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

pub(crate) const SVC_CATEGORY_INIT: u32 = 1;
pub(crate) const SVC_CATEGORY_WIPIC: u32 = 3;
pub(crate) const SVC_CATEGORY_STDLIB: u32 = 5;
