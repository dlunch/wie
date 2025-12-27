use wie_util::Result;

use wipi_types::wipic::WIPICIndirectPtr;

use crate::context::WIPICContext;

pub async fn create_application_context(_context: &mut dyn WIPICContext) -> Result<WIPICIndirectPtr> {
    tracing::warn!("stub MC_uicCreateApplicationContext");

    Ok(WIPICIndirectPtr(0))
}
