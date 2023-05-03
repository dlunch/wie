use std::mem::size_of;

use crate::core::arm::ArmCore;

use super::{java::get_wipi_jb_interface, Context};

#[repr(C)]
#[derive(Clone, Copy)]
struct WIPICKnlInterface {
    pub unk: [u32; 33],
    pub fn_get_wipic_interfaces: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WIPICInterface {
    pub interface_0: u32,
    pub interface_1: u32,
    pub interface_2: u32,
    pub interface_3: u32,
    pub interface_4: u32,
    pub interface_5: u32,
    pub interface_6: u32,
    pub interface_7: u32,
    pub interface_8: u32,
    pub interface_9: u32,
    pub interface_10: u32,
    pub interface_11: u32,
    pub interface_12: u32,
}

pub fn get_interface(core: &mut ArmCore, context: &Context, r#struct: String) -> u32 {
    log::debug!("get_interface({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs().unwrap());

            0
        }
    }
}

fn get_wipic_knl_interface(core: &mut ArmCore, context: &Context) -> u32 {
    let knl_interface = WIPICKnlInterface {
        unk: [0; 33],
        fn_get_wipic_interfaces: core.register_function(get_wipic_interfaces, context).unwrap(),
    };

    let address = context.borrow_mut().allocator.alloc(size_of::<WIPICKnlInterface>() as u32).unwrap();
    core.write(address, knl_interface).unwrap();

    address
}

fn get_wipic_interfaces(core: &mut ArmCore, context: &Context) -> u32 {
    log::debug!("get_wipic_interfaces");

    let interface = WIPICInterface {
        interface_0: 0,
        interface_1: 0,
        interface_2: 0,
        interface_3: 0,
        interface_4: 0,
        interface_5: 0,
        interface_6: 0,
        interface_7: 0,
        interface_8: 0,
        interface_9: 0,
        interface_10: 0,
        interface_11: 0,
        interface_12: 0,
    };

    let address = context.borrow_mut().allocator.alloc(size_of::<WIPICInterface>() as u32).unwrap();

    core.write(address, interface).unwrap();

    address
}
