use crate::emulator::arm::ArmEmulator;

pub fn get_system_function(emulator: &mut ArmEmulator, function: String) -> u32 {
    log::debug!("{}", function);

    log::debug!("\n{}", emulator.dump_regs());

    0
}
