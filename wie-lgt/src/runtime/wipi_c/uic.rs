use wie_util::Result;
use wie_wipi_c::{WIPICContext, api::uic};

use crate::runtime::wipi_c::graphics;

pub async fn create_application_context(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::debug!("MC_uicCreateApplicationContext()");
    graphics::screen_framebuffer_slot(context)
}

pub async fn paint(context: &mut dyn WIPICContext, cc: u32, gc: u32) -> Result<()> {
    uic::paint_with(context, cc, gc, graphics::drawing_target).await
}
