use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody, CResult, CWord},
    method::MethodImpl,
};

fn gen_stub(id: CWord, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented graphics{}: {}", id, name)) };

    body.into_body()
}

async fn htons(_context: &mut dyn CContext, val: CWord) -> CResult<CWord> {
    tracing::warn!("stub MC_utilHtons({})", val);

    Ok((val as u16).to_be() as _) // XXX we're always on little endian
}

pub fn get_util_method_table() -> Vec<CMethodBody> {
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
