use alloc::{boxed::Box, vec::Vec};

use wipi_types::wipic::WIPICWord;

use wie_util::{Result, WieError};

use crate::{WIPICResult, context::WIPICContext, method::MethodBody};

pub async fn connect(context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_netConnect({cb:#x}, {param:#x})");

    struct ConnectCallback {
        cb: WIPICWord,
        param: WIPICWord,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for ConnectCallback {
        #[tracing::instrument(name = "timer", skip_all)]
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            context.system().sleep(1).await; // simulate some delay

            context.call_function(self.cb, &[u32::MAX, self.param]).await?; // callback with M_E_ERROR

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    context.spawn(Box::new(ConnectCallback { cb, param }))?;

    Ok(0)
}

pub async fn close(_context: &mut dyn WIPICContext) -> Result<()> {
    tracing::warn!("stub MC_netClose()");

    Ok(())
}

pub async fn socket_close(_context: &mut dyn WIPICContext, fd: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketClose({fd})");

    Ok(-1) // M_E_ERROR
}

pub async fn socket(_context: &mut dyn WIPICContext, domain: i32, socket_type: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocket({domain}, {socket_type})");
    Ok(-1)
}

pub async fn socket_connect(
    _context: &mut dyn WIPICContext,
    fd: i32,
    address: WIPICWord,
    port: WIPICWord,
    cb: WIPICWord,
    param: WIPICWord,
) -> Result<i32> {
    tracing::warn!("stub MC_netSocketConnect({fd}, {address:#x}, {port}, {cb:#x}, {param:#x})");
    Ok(-1)
}

pub async fn socket_write(_context: &mut dyn WIPICContext, fd: i32, buffer: WIPICWord, length: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketWrite({fd}, {buffer:#x}, {length})");
    Ok(-1)
}

pub async fn socket_read(_context: &mut dyn WIPICContext, fd: i32, buffer: WIPICWord, length: i32) -> Result<i32> {
    tracing::warn!("stub MC_netSocketRead({fd}, {buffer:#x}, {length})");
    Ok(-1)
}

pub async fn set_read_callback(_context: &mut dyn WIPICContext, fd: i32, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_netSetReadCB({fd}, {cb:#x}, {param:#x})");
    Ok(-1)
}
