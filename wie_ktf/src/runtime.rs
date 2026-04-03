mod init;
mod java;
mod svc_ids;
mod wipi_c;

pub(crate) const SVC_CATEGORY_INIT: u32 = 1;
pub(crate) const SVC_CATEGORY_JAVA_INTERFACE: u32 = 2;
pub(crate) const SVC_CATEGORY_WIPIC: u32 = 3;
pub(crate) const SVC_CATEGORY_JAVA: u32 = 4;

pub use self::java::jvm_support::KtfJvmSupport;
