use wipi_types::wipic::WIPICWord;

use wie_util::Result;

use crate::WIPICContext;

pub async fn open(_context: &mut dyn WIPICContext, port: i32, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_srlOpen({port}, {param:#x})");
    Ok(-1)
}

pub async fn write(_context: &mut dyn WIPICContext, fd: i32, buffer: WIPICWord, size: i32) -> Result<i32> {
    tracing::warn!("stub MC_srlWrite({fd}, {buffer:#x}, {size})");
    Ok(-1)
}

pub async fn set_write_callback(_context: &mut dyn WIPICContext, fd: i32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_srlSetWriteCB({fd}, {cb:#x}, {param:#x})");
    Ok(-1)
}

pub async fn close(_context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::warn!("stub MC_srlClose({fd})");
    Ok(-1)
}
