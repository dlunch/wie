use alloc::{vec, vec::Vec};

use crate::{context::WIPICContext, method::MethodImpl, WIPICError, WIPICMethodBody, WIPICResult, WIPICWord};

fn gen_stub(_id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WIPICError::Unimplemented(name.into())) };

    body.into_body()
}

async fn htons(_context: &mut dyn WIPICContext, val: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::debug!("MC_utilHtons({})", val);

    Ok((val as u16).to_be() as _) // XXX we're always on little endian
}

pub fn get_util_method_table() -> Vec<WIPICMethodBody> {
    vec![
        gen_stub(0, "MC_utilHtonl"),
        htons.into_body(),
        gen_stub(2, "MC_utilNtohl"),
        gen_stub(3, "MC_utilNtohs"),
        gen_stub(4, "MC_utilInetAddrInt"),
        gen_stub(5, "MC_utilInetAddrStr"),
        gen_stub(6, "OEMC_utilHashbySHA1"),
    ]
}
