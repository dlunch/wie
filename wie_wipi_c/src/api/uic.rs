use alloc::{vec, vec::Vec};

use wie_util::{Result, WieError};

use crate::{context::WIPICContext, method::MethodImpl, WIPICMemoryId, WIPICMethodBody, WIPICWord};

fn gen_stub(_id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WieError::Unimplemented(name.into())) };

    body.into_body()
}

async fn create_application_context(_context: &mut dyn WIPICContext) -> Result<WIPICMemoryId> {
    tracing::warn!("stub MC_uicCreateApplicationContext");

    Ok(WIPICMemoryId(0))
}

pub fn get_uic_method_table() -> Vec<WIPICMethodBody> {
    vec![
        create_application_context.into_body(),
        gen_stub(1, "MC_uicGetClass"),
        gen_stub(2, "MC_uicCreate"),
        gen_stub(3, "MC_uicDestroy"),
        gen_stub(4, "MC_uicRepaint"),
        gen_stub(5, "MC_uicPaint"),
        gen_stub(6, "MC_uicGetClassName"),
        gen_stub(7, "MC_uicIsInstance"),
        gen_stub(8, "MC_uicHandleEvent"),
        gen_stub(9, "MC_uicConfigure"),
        gen_stub(10, "MC_uicGetGeometry"),
        gen_stub(11, "MC_uicSetEnable"),
        gen_stub(12, "MC_uicSetCallback"),
        gen_stub(13, "MC_uicSetEventHandler"),
        gen_stub(14, "MC_uicSetFont"),
        gen_stub(15, "MC_uicGetFont"),
        gen_stub(16, "MC_uicSetFgColor"),
        gen_stub(17, "MC_uicSetBgColor"),
        gen_stub(18, "MC_uicSetLabel"),
        gen_stub(19, "MC_uicGetLabel"),
        gen_stub(20, "MC_uicSetLabelAlignment"),
        gen_stub(21, "MC_uicSetTimeMask"),
        gen_stub(22, "MC_uicSetTime"),
        gen_stub(23, "MC_uicSetTimeLong"),
        gen_stub(24, "MC_uicGetTime"),
        gen_stub(25, "MC_uicAddMenuItem"),
        gen_stub(26, "MC_uicGetMenuItem"),
        gen_stub(27, "MC_uicRemoveMenuItem"),
        gen_stub(28, "MC_uicSetActiveMenuItem"),
        gen_stub(29, "MC_uicGetActiveMenuItem"),
        gen_stub(30, "MC_uicInsertText"),
        gen_stub(31, "MC_uicDeleteText"),
        gen_stub(32, "MC_uicGetMaxTextSize"),
        gen_stub(33, "MC_uicSetMaxTextSize"),
        gen_stub(34, "MC_uicGetTextSize"),
        gen_stub(35, "MC_uicGetText"),
        gen_stub(36, "MC_uicAddListItem"),
        gen_stub(37, "MC_uicGetListItem"),
        gen_stub(38, "MC_uicRemoveListItem"),
        gen_stub(39, "MC_uicSetActiveListItem"),
        gen_stub(40, "MC_uicGetActiveListItem"),
        gen_stub(41, "OEMC_uicGetCursorPosition"),
        gen_stub(42, "OEMC_uicSetCursorPosition"),
        gen_stub(43, "OEMC_uicSetLineGap"),
        gen_stub(44, "OEMC_uicGetLineGap"),
    ]
}
