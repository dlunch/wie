#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 5] {
    [
        classes::com::nokia::mid::ui::DeviceControl::as_proto(),
        classes::com::nokia::mid::ui::DirectGraphics::as_proto(),
        classes::net::wie::NokiaDirectGraphics::as_proto(),
        classes::com::nokia::mid::ui::DirectUtils::as_proto(),
        classes::com::nokia::mid::ui::FullCanvas::as_proto(),
    ]
}
