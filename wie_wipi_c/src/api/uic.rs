use wie_util::Result;

use crate::{context::WIPICContext, WIPICMemoryId};

pub async fn create_application_context(_context: &mut dyn WIPICContext) -> Result<WIPICMemoryId> {
    tracing::warn!("stub MC_uicCreateApplicationContext");

    Ok(WIPICMemoryId(0))
}
