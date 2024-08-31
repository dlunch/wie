#![no_std]
extern crate alloc;

use wie_jvm_support::WieJavaClassProto;

pub mod classes;

pub fn get_protos() -> [WieJavaClassProto; 1] {
    [classes::com::skt::m::Device::as_proto()]
}
