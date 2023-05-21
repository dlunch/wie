#![no_std]
extern crate alloc;

mod array;
mod base;
mod r#impl;
mod method;
mod proxy;

pub use self::{
    base::{get_array_proto, get_class_proto, JavaClassProto, JavaContext, JavaError, JavaMethodBody, JavaResult},
    proxy::JavaObjectProxy,
};
