use alloc::{vec, vec::Vec};

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICResult, WIPICWord},
    method::MethodImpl,
};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented misc{}: {}", id, name)) };

    body.into_body()
}

async fn back_light(
    _context: &mut dyn WIPICContext,
    id: WIPICWord,
    on_off: WIPICWord,
    color: WIPICWord,
    timeout: WIPICWord,
) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_miscBackLight({}, {}, {}, {})", id, on_off, color, timeout);

    Ok(0)
}

pub fn get_misc_method_table() -> Vec<WIPICMethodBody> {
    vec![
        back_light.into_body(),
        gen_stub(1, "MC_miscSetLed"),
        gen_stub(2, "MC_miscGetLed"),
        gen_stub(3, "MC_miscGetLedCount"),
        gen_stub(4, "OEMC_miscGetCompassData"),
    ]
}
