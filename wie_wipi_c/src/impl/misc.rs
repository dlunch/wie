use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody, CResult, CWord},
    method::MethodImpl,
};

fn gen_stub(id: CWord, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented misc{}: {}", id, name)) };

    body.into_body()
}

async fn back_light(_context: &mut dyn CContext, id: CWord, on_off: CWord, color: CWord, timeout: CWord) -> CResult<CWord> {
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
