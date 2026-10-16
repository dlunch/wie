use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use crate::context::WIPICContext;

pub async fn create_application_context(_context: &mut dyn WIPICContext) -> Result<WIPICIndirectPtr> {
    tracing::warn!("stub MC_uicCreateApplicationContext");

    Ok(WIPICIndirectPtr(0))
}

pub async fn get_class(context: &mut dyn WIPICContext, psz: WIPICWord) -> Result<WIPICIndirectPtr> {
    let name_bytes = read_null_terminated_string_bytes(context, psz)?;
    let name = encoding_rs::EUC_KR.decode(&name_bytes).0;
    tracing::warn!("stub MC_uicGetClass({name})");

    Ok(WIPICIndirectPtr(0))
}

pub async fn create(_context: &mut dyn WIPICContext, pac: WIPICWord, cls: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::warn!("stub MC_uicCreate({pac:#x}, {cls:#x})");

    Ok(WIPICIndirectPtr(0))
}

pub async fn destroy(_context: &mut dyn WIPICContext, cc: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_uicDestroy({cc:#x})");

    Ok(())
}

pub async fn configure(_context: &mut dyn WIPICContext, cc: WIPICWord, x: i32, y: i32, w: i32, h: i32, mask: i32) -> Result<()> {
    tracing::warn!("stub MC_uicConfigure({cc:#x}, {x}, {y}, {w}, {h}, {mask:#x})");

    Ok(())
}

pub async fn set_enable(_context: &mut dyn WIPICContext, cc: WIPICWord, enable: i32) -> Result<()> {
    tracing::warn!("stub MC_uicSetEnable({cc:#x}, {enable})");

    Ok(())
}

pub async fn insert_text(_context: &mut dyn WIPICContext, cc: WIPICWord, idx: i32, psz: WIPICWord, len: i32) -> Result<i32> {
    tracing::warn!("stub MC_uicInsertText({cc:#x}, {idx}, {psz:#x}, {len})");

    Ok(0)
}

pub async fn set_max_text_size(_context: &mut dyn WIPICContext, cc: WIPICWord, max: i32) -> Result<i32> {
    tracing::warn!("stub MC_uicSetMaxTextSize({cc:#x}, {max})");

    Ok(0)
}

pub async fn get_menu_item(_context: &mut dyn WIPICContext, cc: WIPICWord, idx: u32, psz: WIPICWord, buflen: i32, img: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_uicGetMenuItem({cc:#x}, {idx}, {psz:#x}, {buflen}, {img:#x})");

    Ok(0)
}
