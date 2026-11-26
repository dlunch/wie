use wie_util::{Result, write_generic};
use wie_wipi_c::WIPICContext;

pub async fn unk18(_context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::warn!("stub MC_plugUnk18()");

    Ok(0)
}

pub async fn unk19(context: &mut dyn WIPICContext, ptr_callback: u32, a1: u32, a2: u32) -> Result<u32> {
    tracing::warn!("stub MC_plugUnk19({ptr_callback:#x}, {a1:#x}, {a2:#x})");

    // Callers fall back to their own handling when no layout callback is returned.
    write_generic(context, ptr_callback, 0u32)?;

    Ok(0)
}
