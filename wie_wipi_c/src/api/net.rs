use wie_util::Result;

use crate::{context::WIPICContext, WIPICWord};

pub async fn connect(_context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_netConnect({:#x}, {:#x})", cb, param);

    Ok(-1) // M_E_ERROR
}

pub async fn close(_context: &mut dyn WIPICContext) -> Result<()> {
    tracing::warn!("stub MC_netClose()");

    Ok(())
}

pub async fn socket_close(_context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketClose({})", fd);

    Ok(-1) // M_E_ERROR
}
