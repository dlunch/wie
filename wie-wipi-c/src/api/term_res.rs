use wie_util::Result;

use crate::context::WIPICContext;

pub async fn get_supported_groups(_context: &mut dyn WIPICContext, res_group: u32, buf_size: i32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetSupportedGroups({res_group:#x}, {buf_size})");

    Ok(-1) // M_E_ERROR
}

pub async fn get_count(_context: &mut dyn WIPICContext, group: u32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetCount({group:#x})");

    Ok(-1) // M_E_ERROR
}

pub async fn get_list(_context: &mut dyn WIPICContext, group: u32, buffer: u32, buffer_size: i32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetList({group:#x}, {buffer:#x}, {buffer_size})");

    Ok(-1) // M_E_ERROR
}

pub async fn get_format(_context: &mut dyn WIPICContext, group: u32, name: u32, buffer: u32, buffer_size: i32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetFormat({group:#x}, {name:#x}, {buffer:#x}, {buffer_size})");

    Ok(-1) // M_E_ERROR
}

pub async fn get_size(_context: &mut dyn WIPICContext, group: u32, name: u32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetSize({group:#x}, {name:#x})");

    Ok(-1) // M_E_ERROR
}

pub async fn get_ui_name(_context: &mut dyn WIPICContext, group: u32, name: u32, buffer: u32, buffer_size: i32) -> Result<i32> {
    tracing::warn!("stub MC_termResGetUIName({group:#x}, {name:#x}, {buffer:#x}, {buffer_size})");

    Ok(-1) // M_E_ERROR
}
