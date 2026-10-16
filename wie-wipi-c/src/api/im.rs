use wie_util::{Result, write_generic};

use wipi_types::wipic::WIPICWord;

use crate::context::WIPICContext;

pub async fn get_support_mode_count(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_imGetSurpportModeCount()");
    Ok(0)
}

pub async fn get_supported_modes(_context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_imGetSupportedModes()");
    Ok(0)
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
