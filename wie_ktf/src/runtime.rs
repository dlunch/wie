mod init;
mod java;
mod wipi_c;

pub use self::java::{jvm_support::KtfJvmSupport, wipi_context::KtfWIPIJavaContext};

pub type RuntimeResult<T> = anyhow::Result<T>;
