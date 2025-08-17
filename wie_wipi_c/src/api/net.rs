use alloc::{boxed::Box, vec::Vec};

use wie_util::{Result, WieError};

use crate::{WIPICResult, WIPICWord, context::WIPICContext, method::MethodBody};

pub async fn connect(context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_netConnect({:#x}, {:#x})", cb, param);

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
