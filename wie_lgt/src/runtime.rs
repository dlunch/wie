pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

pub(crate) const SVC_CATEGORY_INIT: u32 = 1;
pub(crate) const SVC_CATEGORY_WIPI: u32 = 2;
pub(crate) const SVC_CATEGORY_STDLIB: u32 = 4;
