#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod array;
mod base;
mod r#impl;
mod method;
mod proxy;

pub use self::{
    base::{get_array_proto, get_class_proto, JavaAccessFlag, JavaClassProto, JavaContext, JavaError, JavaMethodBody, JavaResult},
    proxy::JavaObjectProxy,
};
