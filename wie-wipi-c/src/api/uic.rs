use wie_util::{Result, read_null_terminated_string_bytes, write_generic};

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

pub async fn get_time(context: &mut dyn WIPICContext, component: u32, out_ptr: u32) -> Result<()> {
    tracing::warn!("stub MC_uicGetTime({component:#x}, {out_ptr:#x})");

    if out_ptr == 0 {
        return Ok(());
    }
    // shortcut: return current UTC time until UI components retain their own date/time state.
    let timestamp = context.system().platform().now().raw() / 1000;
    let (year, month, day, hour, minute, second) = unix_seconds_to_utc(timestamp as i64);
    write_generic(context, out_ptr, second)?;
    write_generic(context, out_ptr + 4, minute)?;
    write_generic(context, out_ptr + 8, hour)?;
    write_generic(context, out_ptr + 12, day)?;
    write_generic(context, out_ptr + 16, month - 1)?;
    write_generic(context, out_ptr + 20, year - 1900)?;

    Ok(())
}

fn unix_seconds_to_utc(timestamp: i64) -> (i32, i32, i32, i32, i32, i32) {
    let days = timestamp.div_euclid(86_400);
    let seconds_of_day = timestamp.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (seconds_of_day / 3600) as i32;
    let minute = ((seconds_of_day % 3600) / 60) as i32;
    let second = (seconds_of_day % 60) as i32;

    (year, month, day, hour, minute, second)
}

fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_param = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_param + 2) / 5 + 1;
    let month = month_param + if month_param < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year as i32, month as i32, day as i32)
}
