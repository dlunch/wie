use std::mem::size_of;

use crate::core::arm::ArmCore;

use super::{
    context::Context,
    types::{WIPICInterface, WIPICKnlInterface},
};

pub fn get_system_struct(core: &mut ArmCore, context: &Context, r#struct: String) -> u32 {
    log::debug!("get_system_struct({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs().unwrap());

            0
        }
    }
}

pub fn init_unk1(core: &mut ArmCore, context: &Context, a0: u32, a1: String) -> u32 {
    // java instantiate?
    log::debug!("init_unk1({}, {})", a0, a1);

    log::debug!("\n{}", core.dump_regs().unwrap());

    let address = (*context).borrow_mut().allocator.alloc(0x20).unwrap();
    core.write(address, [0, 0, 1234, 0, 0, 0, 0, 0, 0]).unwrap();
    core.write(a0, address).unwrap();

    0
}

pub fn init_unk2(core: &mut ArmCore, context: &Context, a0: u32, a1: u32) -> u32 {
    // calloc??
    log::debug!("init_unk2({}, {})", a0, a1);

    log::debug!("\n{}", core.dump_regs().unwrap());

    (*context).borrow_mut().allocator.alloc(a0 * a1).unwrap()
}

fn get_wipic_knl_interface(core: &mut ArmCore, context: &Context) -> u32 {
    let knl_interface = WIPICKnlInterface {
        unk: [0; 33],
        fn_get_interfaces: core.register_function(get_wipic_interfaces, context).unwrap(),
    };

    let address = (*context).borrow_mut().allocator.alloc(size_of::<WIPICKnlInterface>() as u32).unwrap();

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

    let address = (*context).borrow_mut().allocator.alloc(size_of::<WIPICInterface>() as u32).unwrap();

    core.write(address, interface).unwrap();

    address
}
