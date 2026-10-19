use wie_util::Result;

use crate::context::WIPICContext;

pub async fn list(_context: &mut dyn WIPICContext, name: u32, buffer: u32, buffer_size: i32, access_mode: i32) -> Result<i32> {
    tracing::warn!("stub MC_fsList({name:#x}, {buffer:#x}, {buffer_size}, {access_mode})");

    Ok(-1) // M_E_ERROR
}
