use std::rc::Rc;

use crate::core::arm::ArmCore;

use super::{context::Context, types::WIPICKnlInterface};

pub fn get_system_struct(core: &mut ArmCore, context: &Rc<Context>, r#struct: String) -> u32 {
    log::debug!("get_system_struct {}", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs());

            0
        }
    }
}

fn get_wipic_knl_interface(core: &mut ArmCore, context: &Rc<Context>) -> u32 {
    let interface = WIPICKnlInterface {
        unk: [0; 33],
        get_interfaces_fn: core.register_function(get_wipic_interfaces, context),
    };

    core.write(0x40000100, interface);

    0x40000100
}

fn get_wipic_interfaces(core: &mut ArmCore, _context: &Rc<Context>) -> u32 {
    log::debug!("get_wipic_interfaces");

    log::debug!("Register dump\n{}", core.dump_regs());
    0
}
