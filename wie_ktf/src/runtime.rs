mod init;
mod java;
mod wipi_c;

pub use self::{
    init::{
        KtfPeb, {init, start},
    },
    java::wipi_context::KtfWIPIJavaContext,
};

pub type RuntimeResult<T> = anyhow::Result<T>;
