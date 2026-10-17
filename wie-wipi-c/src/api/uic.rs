use alloc::vec;

use bytemuck::{Pod, Zeroable};
use wie_backend::canvas::{Clip, PixelType, Rgb8Pixel};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use crate::{
    api::{
        graphics::{FrameBuffer, primitives},
        im,
    },
    context::WIPICContext,
    globals::UIC_TEXT_CLASS,
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct TextComponent {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    enabled: i32,
    foreground: u32,
    background: u32,
    text: u32,
    length: u32,
    maximum: u32,
    cursor: u32,
}

pub async fn create_application_context(context: &mut dyn WIPICContext) -> Result<WIPICIndirectPtr> {
    let address = context.alloc_raw(4)?;
    write_generic(context, address, 0u32)?;
    Ok(WIPICIndirectPtr(address))
}

pub async fn get_class(context: &mut dyn WIPICContext, psz: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_uicGetClass({psz:#x})");
    let name_bytes = read_null_terminated_string_bytes(context, psz)?;
    if name_bytes != b"TextComponent" {
        return Ok(WIPICIndirectPtr(u32::MAX));
    }
    if read_generic::<u32, _>(context, UIC_TEXT_CLASS)? == 0 {
        write_generic(context, UIC_TEXT_CLASS, 1u32)?;
    }
    Ok(WIPICIndirectPtr(UIC_TEXT_CLASS))
}

pub async fn create(context: &mut dyn WIPICContext, pac: WIPICWord, cls: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_uicCreate({pac:#x}, {cls:#x})");
    if pac == 0 || cls == 0 || cls == u32::MAX {
        return Ok(WIPICIndirectPtr(u32::MAX));
    }
    let text = match context.alloc_raw(257) {
        Ok(text) => text,
        Err(WieError::AllocationFailure) => return Ok(WIPICIndirectPtr(u32::MAX)),
        Err(error) => return Err(error),
    };
    context.write_bytes(text, &[0])?;
    let component = TextComponent {
        background: 0xffffff,
        text,
        maximum: 256,
        ..TextComponent::zeroed()
    };
    let address = match context.alloc_raw(size_of::<TextComponent>() as u32) {
        Ok(address) => address,
        Err(error) => {
            context.free_raw(text, 257)?;
            return match error {
                WieError::AllocationFailure => Ok(WIPICIndirectPtr(u32::MAX)),
                error => Err(error),
            };
        }
    };
    write_generic(context, address, component)?;
    Ok(WIPICIndirectPtr(address))
}

pub async fn destroy(context: &mut dyn WIPICContext, cc: WIPICWord) -> Result<()> {
    let component: TextComponent = read_generic(context, cc)?;
    context.free_raw(component.text, component.maximum + 1)?;
    context.free_raw(cc, size_of::<TextComponent>() as u32)
}

pub async fn configure(context: &mut dyn WIPICContext, cc: WIPICWord, x: i32, y: i32, w: i32, h: i32, mask: i32) -> Result<()> {
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let mut component: TextComponent = read_generic(context, cc)?;
    if mask & 1 != 0 {
        component.x = x;
        component.y = y;
    }
    if mask & 2 != 0 {
        component.width = w;
        component.height = h;
    }
    write_generic(context, cc, component)
}

pub async fn set_enable(context: &mut dyn WIPICContext, cc: WIPICWord, enable: i32) -> Result<()> {
    let mut component: TextComponent = read_generic(context, cc)?;
    component.enabled = enable;
    write_generic(context, cc, component)
}

pub async fn insert_text(context: &mut dyn WIPICContext, cc: WIPICWord, idx: i32, psz: WIPICWord, len: i32) -> Result<i32> {
    let mut component: TextComponent = read_generic(context, cc)?;
    let count = (len.max(0) as u32).min(component.maximum - component.length);
    let index = (idx.max(0) as u32).min(component.length);
    let mut inserted = vec![0; count as usize];
    context.read_bytes(psz, &mut inserted)?;
    let mut tail = vec![0; (component.length - index + 1) as usize];
    context.read_bytes(component.text + index, &mut tail)?;
    context.write_bytes(component.text + index + count, &tail)?;
    context.write_bytes(component.text + index, &inserted)?;
    component.length += count;
    component.cursor = index + count;
    write_generic(context, cc, component)?;
    Ok(count as i32)
}

pub async fn set_max_text_size(context: &mut dyn WIPICContext, cc: WIPICWord, max: i32) -> Result<i32> {
    tracing::debug!("MC_uicSetMaxTextSize({cc:#x}, {max})");
    let mut component: TextComponent = read_generic(context, cc)?;
    let previous = component.maximum;
    if max < 0 || max as u32 == previous {
        return Ok(previous as i32);
    }
    let maximum = max as u32;
    let length = component.length.min(maximum);
    let mut bytes = vec![0; length as usize + 1];
    context.read_bytes(component.text, &mut bytes[..length as usize])?;
    let text = match context.alloc_raw(maximum + 1) {
        Ok(text) => text,
        Err(WieError::AllocationFailure) => return Ok(-17),
        Err(error) => return Err(error),
    };
    context.write_bytes(text, &bytes)?;
    context.free_raw(component.text, previous + 1)?;
    component.text = text;
    component.maximum = maximum;
    component.length = length;
    component.cursor = component.cursor.min(length);
    write_generic(context, cc, component)?;
    Ok(previous as i32)
}

pub async fn delete_text(context: &mut dyn WIPICContext, cc: u32, idx: i32, len: i32) -> Result<()> {
    tracing::debug!("MC_uicDeleteText({cc:#x}, {idx}, {len})");
    let mut component: TextComponent = read_generic(context, cc)?;
    if idx < 0 || idx as u32 >= component.length || len < -1 {
        return Ok(());
    }
    let index = idx as u32;
    let count = if len == -1 {
        component.length - index
    } else {
        (len as u32).min(component.length - index)
    };
    let mut tail = vec![0; (component.length - index - count + 1) as usize];
    context.read_bytes(component.text + index + count, &mut tail)?;
    context.write_bytes(component.text + index, &tail)?;
    component.length -= count;
    if component.cursor > index {
        component.cursor -= count.min(component.cursor - index);
    }
    write_generic(context, cc, component)
}

pub async fn get_text_size(context: &mut dyn WIPICContext, cc: u32) -> Result<i32> {
    let component: TextComponent = read_generic(context, cc)?;
    Ok(component.length as i32)
}

pub async fn get_text(context: &mut dyn WIPICContext, cc: u32, idx: i32, destination: u32, len: i32) -> Result<i32> {
    tracing::debug!("MC_uicGetText({cc:#x}, {idx}, {destination:#x}, {len})");
    let component: TextComponent = read_generic(context, cc)?;
    let index = idx.max(0) as u32;
    if index > component.length || len <= 0 {
        return Ok(0);
    }
    let count = (component.length - index).min(len as u32);
    let mut bytes = vec![0; count as usize + usize::from(count < len as u32)];
    context.read_bytes(component.text + index, &mut bytes[..count as usize])?;
    context.write_bytes(destination, &bytes)?;
    Ok(count as i32)
}

pub async fn set_fg_color(context: &mut dyn WIPICContext, cc: u32, color: u32) -> Result<()> {
    let mut component: TextComponent = read_generic(context, cc)?;
    component.foreground = color;
    write_generic(context, cc, component)
}

pub async fn set_bg_color(context: &mut dyn WIPICContext, cc: u32, color: u32) -> Result<()> {
    let mut component: TextComponent = read_generic(context, cc)?;
    component.background = color;
    write_generic(context, cc, component)
}

pub async fn set_label_alignment(_context: &mut dyn WIPICContext, _cc: u32, _align: i32) -> Result<i32> {
    // KWISF specifies no effect for non-label components; only TextComponent is supported.
    Ok(0)
}

pub async fn paint_component(
    context: &mut dyn WIPICContext,
    cc: u32,
    framebuffer: &FrameBuffer,
    clip: Clip,
    translation_x: i32,
    translation_y: i32,
) -> Result<()> {
    tracing::debug!("MC_uicPaint({cc:#x})");
    let component: TextComponent = read_generic(context, cc)?;
    if component.width <= 0 || component.height <= 0 {
        return Ok(());
    }
    let x = component.x.saturating_add(translation_x);
    let y = component.y.saturating_add(translation_y);
    let clip = clip.intersect(&Clip {
        x,
        y,
        width: component.width as u32,
        height: component.height as u32,
    });
    if clip.width == 0 || clip.height == 0 {
        return Ok(());
    }
    primitives::fill_rect(
        context,
        framebuffer,
        x,
        y,
        component.width as u32,
        component.height as u32,
        Rgb8Pixel::to_color(component.background),
        clip,
    )?;
    let mut bytes = vec![0; component.length as usize];
    context.read_bytes(component.text, &mut bytes)?;
    let text = encoding_rs::EUC_KR.decode(&bytes).0;
    primitives::draw_text(context, framebuffer, &text, x, y, Rgb8Pixel::to_color(component.foreground), clip)
}

pub async fn handle_event(context: &mut dyn WIPICContext, cc: u32, event_type: i32, key: i32, _param2: i32) -> Result<i32> {
    tracing::debug!("MC_uicHandleEvent({cc:#x}, {event_type}, {key}, {_param2})");
    let mut component: TextComponent = read_generic(context, cc)?;
    if component.enabled == 0 || !matches!(event_type, 2..=4) {
        return Ok(0);
    }
    if event_type != 3 && matches!(key, -3 | -4 | -16) {
        let mut bytes = vec![0; component.length as usize];
        context.read_bytes(component.text, &mut bytes)?;
        // Walk encoded character boundaries so cursor movement cannot split an EUC-KR character.
        let mut previous = 0;
        let mut next = 0;
        while next < component.cursor as usize {
            previous = next;
            next += if bytes[next] >= 0x80 { 2 } else { 1 };
        }
        match key {
            -3 => component.cursor = previous as u32,
            -4 if component.cursor < component.length => {
                component.cursor = (component.cursor + if bytes[component.cursor as usize] >= 0x80 { 2 } else { 1 }).min(component.length);
            }
            -16 if component.cursor > 0 => {
                delete_text(context, cc, previous as i32, (component.cursor as usize - previous) as i32).await?;
                return Ok(1);
            }
            _ => {}
        }
        write_generic(context, cc, component)?;
        return Ok(1);
    }
    let scratch = context.alloc_raw(520)?;
    write_generic(context, scratch + 512, 256i32)?;
    write_generic(context, scratch + 516, 256i32)?;
    let result = async {
        if event_type != 3 && (0x30..=0x39).contains(&key) && im::get_current_mode(context).await? == 3 {
            context.write_bytes(scratch, &[key as u8])?;
            insert_text(context, cc, component.cursor as i32, scratch, 1).await?;
            return Ok(1);
        }
        // shortcut: shared IM does not yet compose Latin/Hangul text; consume its committed output when implemented.
        let handled = im::handle_input(context, key as u32, event_type, scratch, scratch + 512, scratch + 256, scratch + 516).await?;
        let committed: i32 = read_generic(context, scratch + 512)?;
        if committed > 0 {
            insert_text(context, cc, component.cursor as i32, scratch, committed.min(256)).await?;
        }
        Ok(handled)
    }
    .await;
    context.free_raw(scratch, 520)?;
    result
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

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite};

    use crate::context::test::TestContext;

    use super::*;

    #[futures_test::test]
    async fn text_component_edits_encoded_text_and_enforces_byte_limits() -> Result<()> {
        let system = System::new(Box::new(TestPlatform::new()), "test", "test", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        context.write_bytes(0x1000, b"TextComponent\0")?;
        let application = create_application_context(&mut context).await?;
        let class = get_class(&mut context, 0x1000).await?;
        assert_eq!(class.0, UIC_TEXT_CLASS);
        assert_eq!(get_class(&mut context, 0x1000).await?.0, class.0);
        assert_eq!(read_generic::<u32, _>(&context, class.0)?, 1);
        let cc = create(&mut context, application.0, class.0).await?.0;
        configure(&mut context, cc, 2, 3, 40, 20, 3).await?;
        configure(&mut context, cc, 4, 5, 99, 99, 1).await?;
        set_enable(&mut context, cc, 1).await?;
        set_fg_color(&mut context, cc, 0xff0000).await?;
        set_bg_color(&mut context, cc, 0xffffff).await?;
        assert_eq!(set_label_alignment(&mut context, cc, 0).await?, 0);
        context.write_bytes(0x1100, b"A\xb0\xa1B")?;
        assert_eq!(insert_text(&mut context, cc, -1, 0x1100, 4).await?, 4);
        context.write_bytes(0x1100, b"xxxx")?;
        assert_eq!(handle_event(&mut context, cc, 2, -3, 0).await?, 1);
        assert_eq!(handle_event(&mut context, cc, 2, -16, 0).await?, 1);
        assert_eq!(get_text_size(&mut context, cc).await?, 2);
        context.write_bytes(0x1200, &[0xcc; 5])?;
        assert_eq!(get_text(&mut context, cc, 0, 0x1200, 3).await?, 2);
        let mut bytes = [0; 5];
        context.read_bytes(0x1200, &mut bytes)?;
        assert_eq!(&bytes, b"AB\0\xcc\xcc");
        assert_eq!(set_max_text_size(&mut context, cc, 3).await?, 256);
        assert_eq!(insert_text(&mut context, cc, 100, 0x1100, 4).await?, 1);
        assert_eq!(get_text_size(&mut context, cc).await?, 3);
        delete_text(&mut context, cc, 2, -1).await?;
        assert_eq!(get_text_size(&mut context, cc).await?, 2);
        context.write_bytes(0x1200, &[0xcc; 5])?;
        assert_eq!(get_text(&mut context, cc, -1, 0x1200, 2).await?, 2);
        context.read_bytes(0x1200, &mut bytes)?;
        assert_eq!(&bytes, b"AB\xcc\xcc\xcc");
        assert_eq!(get_text(&mut context, cc, 10, 0x1200, 3).await?, 0);

        set_enable(&mut context, cc, 0).await?;
        assert_eq!(handle_event(&mut context, cc, 2, -16, 0).await?, 0);
        assert_eq!(set_max_text_size(&mut context, cc, 1).await?, 3);
        assert_eq!(get_text_size(&mut context, cc).await?, 1);
        destroy(&mut context, cc).await?;
        Ok(())
    }
}
