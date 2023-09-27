use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody, CResult},
    method::MethodImpl,
};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented graphics{}: {}", id, name)) };

    body.into_body()
}

async fn back_light(_context: &mut dyn CContext, id: u32, on_off: u32, color: u32, timeout: u32) -> CResult<u32> {
    tracing::warn!("stub MC_miscBackLight({}, {}, {}, {})", id, on_off, color, timeout);

    Ok(0)
}

pub fn get_misc_method_table() -> Vec<CMethodBody> {
    vec![
        back_light.into_body(),
        gen_stub(1, "MC_miscSetLed"),
        gen_stub(2, "MC_miscGetLed"),
        gen_stub(3, "MC_miscGetLedCount"),
        gen_stub(4, "OEMC_miscGetCompassData"),
    ]
}
