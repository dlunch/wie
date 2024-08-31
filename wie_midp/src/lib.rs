#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 1] {
    [classes::javax::microedition::midlet::MIDlet::as_proto()]
}
