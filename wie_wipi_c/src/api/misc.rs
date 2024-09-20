use wie_util::Result;

use crate::{context::WIPICContext, WIPICWord};

pub async fn back_light(
    _context: &mut dyn WIPICContext,
    id: WIPICWord,
    on_off: WIPICWord,
    color: WIPICWord,
    timeout: WIPICWord,
) -> Result<WIPICWord> {
    tracing::warn!("stub MC_miscBackLight({}, {}, {}, {})", id, on_off, color, timeout);

    Ok(0)
}
