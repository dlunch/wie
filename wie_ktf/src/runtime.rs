mod init;
mod java;
mod svc_ids;
mod wipi_c;

pub(crate) const SVC_CATEGORY_INIT: u32 = 1;
pub(crate) const SVC_CATEGORY_WIPI: u32 = 2;
pub(crate) const SVC_CATEGORY_JAVA: u32 = 3;

pub use self::java::jvm_support::KtfJvmSupport;
