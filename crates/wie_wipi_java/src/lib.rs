#![no_std]
extern crate alloc;

mod array;
mod r#impl;
mod base;
mod proxy;

pub use self::{
    base::{get_array_proto, get_class_proto, JavaClassProto, JavaContextBase, JavaError, JavaMethodBody, JavaResult},
    proxy::JavaObjectProxy,
};
