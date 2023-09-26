#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod array;
mod base;
pub mod r#impl;
mod method;
mod proxy;

pub use self::{
    array::Array,
    base::{get_class_proto, JavaClassProto, JavaContext, JavaError, JavaFieldAccessFlag, JavaMethodBody, JavaMethodFlag, JavaResult},
    proxy::JavaObjectProxy,
};
