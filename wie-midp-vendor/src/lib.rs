#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 2] {
    [
        classes::com::nokia::mid::ui::DeviceControl::as_proto(),
        classes::com::nokia::mid::ui::FullCanvas::as_proto(),
    ]
}
