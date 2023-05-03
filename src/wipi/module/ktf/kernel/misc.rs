use crate::core::arm::ArmCore;

use super::Context;

pub fn init_unk3(core: &mut ArmCore, context: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    // calloc??
    log::debug!("init_unk3({}, {})", a0, a1);

    log::debug!("\n{}", core.dump_regs()?);

    context.alloc(a0 * a1)
}
