use crate::emulator::arm::ArmEmulator;

use super::types::WIPICKnlInterface;

pub fn get_system_struct(emulator: &mut ArmEmulator, r#struct: String) -> u32 {
    log::debug!("get_system_struct {}", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(emulator),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", emulator.dump_regs());

            0
        }
    }
}

fn get_wipic_knl_interface(emulator: &mut ArmEmulator) -> u32 {
    let interface = WIPICKnlInterface {
        unk: [0; 33],
        get_interfaces_fn: emulator.register_function(get_wipic_interfaces),
    };

    emulator.write(0x40000100, interface);

    0x40000100
}

fn get_wipic_interfaces(emulator: &mut ArmEmulator) -> u32 {
    log::debug!("get_wipic_interfaces");

    log::debug!("Register dump\n{}", emulator.dump_regs());
    0
}
