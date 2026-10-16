use wie_util::{Result, read_generic, write_generic};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

const INPUT_MODES_PTR: u32 = 0x7fff1020;
const INPUT_MODES: [&[u8]; 4] = [b"EN/L\0", b"EN/S\0", b"KO\0", b"N123\0"];

pub async fn get_support_mode_count(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_imGetSurpportModeCount()");
    Ok(INPUT_MODES.len() as i32)
}

pub async fn get_supported_modes(context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_imGetSupportedModes()");

    let modes: u32 = read_generic(context, INPUT_MODES_PTR)?;
    if modes != 0 {
        return Ok(modes);
    }

    let table_size = (INPUT_MODES.len() * size_of::<u32>()) as u32;
    let strings_size = INPUT_MODES.iter().map(|mode| mode.len() as u32).sum::<u32>();
    let modes = context.alloc_raw(table_size + strings_size)?;
    let mut string_ptr = modes + table_size;
    for (index, mode) in INPUT_MODES.iter().enumerate() {
        write_generic(context, modes + (index * size_of::<u32>()) as u32, string_ptr)?;
        context.write_bytes(string_ptr, mode)?;
        string_ptr += mode.len() as u32;
    }
    write_generic(context, INPUT_MODES_PTR, modes)?;
    Ok(modes)
}

pub async fn set_current_mode(_context: &mut dyn WIPICContext, mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_imSetCurrentMode({mode})");
    Ok(-1)
}

pub async fn get_current_mode(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_imGetCurrentMode()");
    Ok(-1)
}

pub async fn handle_input(
    context: &mut dyn WIPICContext,
    key: WIPICWord,
    event_type: i32,
    ptr_committed: WIPICWord,
    ptr_committed_size: WIPICWord,
    ptr_composing: WIPICWord,
    ptr_composing_size: WIPICWord,
) -> Result<i32> {
    tracing::warn!(
        "stub MC_imHandleInput({key:#x}, {event_type}, {ptr_committed:#x}, {ptr_committed_size:#x}, {ptr_composing:#x}, {ptr_composing_size:#x})"
    );

    write_generic(context, ptr_committed_size, 0i32)?;
    write_generic(context, ptr_composing_size, 0i32)?;

    Ok(0)
}
