use wie_util::Result;

use crate::context::WIPICContext;

pub async fn get_supported_groups(_context: &mut dyn WIPICContext, res_group: u32, buf_size: i32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetSupportedGroups({res_group:#x}, {buf_size})");

    Ok(-1) // M_E_ERROR
}
