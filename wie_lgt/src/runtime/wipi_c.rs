use alloc::format;

mod context;

use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_util::{read_generic, Result, WieError};
use wie_wipi_c::{
    api::{database, graphics, kernel, misc},
    MethodImpl, WIPICContext,
};

use context::LgtWIPICContext;

pub fn get_wipi_c_method(core: &mut ArmCore, system: &mut System, function_index: u32) -> Result<u32> {
    let method = match function_index {
        0x03 => clet_register.into_body(),
        0x64 => kernel::printk.into_body(),
        0x65 => kernel::sprintk.into_body(),
        0x75 => kernel::alloc.into_body(),
        0x77 => kernel::free.into_body(),
        0x79 => kernel::get_free_memory.into_body(),
        0x7e => kernel::get_system_property.into_body(),
        0x7f => kernel::set_system_property.into_body(),
        0x80 => kernel::get_resource_id.into_body(),
        0xe3 => graphics::get_font.into_body(),
        0xe4 => graphics::get_font_height.into_body(),
        0x190 => database::open_database.into_body(),
        0x4b9 => unk0.into_body(),
        0x578 => misc::back_light.into_body(),
        _ => return Err(WieError::FatalError(format!("Unknown lgt wipi import: {:#x}", function_index))),
    };

    let mut context = LgtWIPICContext::new(core.clone(), system.clone());
    // lgt app calls get method only once per function, so it's okay to register function every time
    let address = context.register_function(method)?;

    Ok(address)
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct CletFunctions {
    start_clet: u32,
    pause_clet: u32,
    resume_clet: u32,
    destroy_clet: u32,
    paint_clet: u32,
    handle_input: u32,
}

async fn clet_register(context: &mut dyn WIPICContext, function_table: u32, a1: u32) -> Result<()> {
    tracing::debug!("clet_register({:#x}, {:#x})", function_table, a1);

    let functions: CletFunctions = read_generic(context, function_table)?;
    tracing::info!("CletFunctions: {:x?}", functions);

    context.call_function(functions.start_clet, &[]).await?;

    Ok(())
}

async fn unk0(_context: &mut dyn WIPICContext, a0: u32, a1: u32) -> Result<()> {
    tracing::debug!("unk0({:#x}, {:#x})", a0, a1);

    // maps to OEMC_mdaClipGetInfo on ktf but seems wrong

    Ok(())
}
