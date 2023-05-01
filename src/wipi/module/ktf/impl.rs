use std::mem::size_of;

use crate::core::arm::ArmCore;

use super::{context::Context, types::WIPICKnlInterface};

pub fn get_system_struct(core: &mut ArmCore, context: &Context, r#struct: String) -> u32 {
    log::debug!("get_system_struct {}", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs().unwrap());

            0
        }
    }
}

fn get_wipic_knl_interface(core: &mut ArmCore, context: &Context) -> u32 {
    let interface = WIPICKnlInterface {
        unk: [0; 33],
        get_interfaces_fn: core.register_function(get_wipic_interfaces, context).unwrap(),
    };

    let address = (*context).borrow_mut().allocator.alloc(size_of::<WIPICKnlInterface>() as u32).unwrap();

    core.write(address, interface).unwrap();

    address
}

fn get_wipic_interfaces(core: &mut ArmCore, _context: &Context) -> u32 {
    log::debug!("get_wipic_interfaces");

    log::debug!("Register dump\n{}", core.dump_regs().unwrap());
    0
}
