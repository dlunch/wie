use alloc::{vec, vec::Vec};

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICResult, WIPICWord},
    method::MethodImpl,
};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented net{}: {}", id, name)) };

    body.into_body()
}

async fn connect(_context: &mut dyn WIPICContext, cb: WIPICWord, param: WIPICWord) -> WIPICResult<i32> {
    tracing::warn!("stub MC_netConnect({:#x}, {:#x})", cb, param);

    Ok(-1) // M_E_ERROR
}

async fn close(_context: &mut dyn WIPICContext) -> WIPICResult<()> {
    tracing::warn!("stub MC_netClose()");

    Ok(())
}

pub fn get_net_method_table() -> Vec<WIPICMethodBody> {
    vec![
        connect.into_body(),
        close.into_body(),
        gen_stub(2, "MC_netSocket"),
        gen_stub(3, "MC_netSocketConnect"),
        gen_stub(4, "MC_netSocketWrite"),
        gen_stub(5, "MC_netSocketRead"),
        gen_stub(6, "MC_netSocketClose"),
        gen_stub(7, "MC_netSocketBind"),
        gen_stub(8, "MC_netGetMaxPacketLength"),
        gen_stub(9, "MC_netSocketSendTo"),
        gen_stub(10, "MC_netSocketRcvFrom"),
        gen_stub(11, "MC_netGetHostAddr"),
        gen_stub(12, "MC_netSocketAccept"),
        gen_stub(13, "MC_netSetReadCB"),
        gen_stub(14, "MC_netSetWriteCB"),
        gen_stub(15, "MC_netHttpOpen"),
        gen_stub(16, "MC_netHttpConnect"),
        gen_stub(17, "MC_netHttpSetRequestMethod"),
        gen_stub(18, "MC_netHttpGetRequestMethod"),
        gen_stub(19, "MC_netHttpSetRequestProperty"),
        gen_stub(20, "MC_netHttpGetRequestProperty"),
        gen_stub(21, "MC_netHttpSetProxy"),
        gen_stub(22, "MC_netHttpGetProxy"),
        gen_stub(23, "MC_netHttpGetResponseCode"),
        gen_stub(24, "MC_netHttpGetResponseMessage"),
        gen_stub(25, "MC_netHttpGetHeaderField"),
        gen_stub(26, "MC_netHttpGetLength"),
        gen_stub(27, "MC_netHttpGetType"),
        gen_stub(28, "MC_netHttpGetEncoding"),
        gen_stub(29, "MC_netHttpClose"),
    ]
}
