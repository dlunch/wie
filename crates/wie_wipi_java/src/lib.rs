#![no_std]
#![allow(unknown_lints)]
#![allow(clippy::needless_pass_by_ref_mut)]
extern crate alloc;

mod array;
mod base;
mod r#impl;
mod method;
mod proxy;
mod string;

pub use self::{
    base::{
        get_array_proto, get_class_proto, JavaClassProto, JavaContext, JavaError, JavaFieldAccessFlag, JavaMethodAccessFlag, JavaMethodBody,
        JavaResult,
    },
    proxy::JavaObjectProxy,
    string::{from_java_string, to_java_string},
};
