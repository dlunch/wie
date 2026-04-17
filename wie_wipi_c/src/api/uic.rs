use alloc::string::String;

use wie_util::{Result, read_null_terminated_string_bytes};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use crate::context::WIPICContext;

pub async fn create_application_context(_context: &mut dyn WIPICContext) -> Result<WIPICIndirectPtr> {
    tracing::warn!("stub MC_uicCreateApplicationContext");

    Ok(WIPICIndirectPtr(0))
}

pub async fn get_class(context: &mut dyn WIPICContext, psz: WIPICWord) -> Result<WIPICIndirectPtr> {
    let name = String::from_utf8(read_null_terminated_string_bytes(context, psz)?).unwrap();
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

pub async fn get_menu_item(_context: &mut dyn WIPICContext, cc: WIPICWord, idx: u32, psz: WIPICWord, buflen: i32, img: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_uicGetMenuItem({cc:#x}, {idx}, {psz:#x}, {buflen}, {img:#x})");

    Ok(0)
}
