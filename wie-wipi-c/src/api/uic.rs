use alloc::{vec, vec::Vec};
use core::mem::{offset_of, replace};

use bytemuck::{Pod, Zeroable};
use encoding_rs::EUC_KR;
use wipi_types::wipic::{WIPICGraphicsContext, WIPICIndirectPtr};

use wie_backend::{
    Event,
    canvas::{Clip, FONT_SIZE, PixelType, Rgb8Pixel, TextAlignment, string_width},
};
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use crate::{
    api::{
        graphics::{self, FrameBuffer},
        im,
    },
    context::WIPICContext,
    globals::{SCREEN_FRAMEBUFFER_PTR, UIC_INPUT_COMPONENT, UIC_LAST_DESTROYED, UIC_TEXT_CLASS},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ComponentClass {
    name: u32,
    super_class: u32,
    size: i32,
    create: u32,
    destroy: u32,
    paint: u32,
    handle_event: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Callback {
    index: u32,
    function: u32,
    data: u32,
}

impl Callback {
    async fn invoke(self, context: &mut dyn WIPICContext, cc: u32, server_data: u32) -> Result<()> {
        if self.function != 0 {
            context.call_function(self.function, &[cc, server_data, self.data]).await?;
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextComponent {
    // Public _MC_UicComponent header from MCuic.h.
    class: u32,
    screen: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    mask: u32,
    font: i32,
    foreground: u32,
    background: u32,
    handler: u32,
    callback_table: u32,
    callback_count: u32,
    ext_data: i32,
    // Native text handlers address the cursor, adjacent halfwords, length, and buffer handle by offset.
    cursor: i16,
    reserved: [i16; 3],
    maximum: i16,
    size: i16,
    buffer: u32,
    callbacks: [Callback; 5],
    lost_focus: u32,
}

impl TextComponent {
    fn text(&self, context: &dyn WIPICContext) -> Result<Vec<u8>> {
        let mut bytes = vec![0; self.size as usize];
        context.read_bytes(context.data_ptr(WIPICIndirectPtr(self.buffer))?, &mut bytes)?;
        Ok(bytes)
    }

    async fn store(&mut self, context: &mut dyn WIPICContext, cc: u32, text: &[u8], cursor: usize) -> Result<()> {
        write_null_terminated_string_bytes(context, context.data_ptr(WIPICIndirectPtr(self.buffer))?, text)?;
        self.size = text.len() as i16;
        self.cursor = cursor as i16;
        let address = context.data_ptr(WIPICIndirectPtr(cc))?;
        write_generic(context, address, *self)?;
        context.system().event_queue().push(Event::Repaint);
        self.callbacks[3].invoke(context, cc, 0).await
    }
}

// EUC-KR trail bytes can also be ASCII. Walk from a known character boundary.
fn boundary(bytes: &[u8], limit: usize) -> usize {
    let limit = limit.min(bytes.len());
    let mut pos = 0;
    while pos < limit {
        let next = pos + if bytes[pos].is_ascii() { 1 } else { 2 };
        if next > limit {
            break;
        }
        pos = next;
    }
    pos
}

pub async fn create_application_context(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::debug!("MC_uicCreateApplicationContext()");
    graphics::get_screen_framebuffer(context, 0).await?;
    Ok(SCREEN_FRAMEBUFFER_PTR)
}

pub async fn get_class(context: &mut dyn WIPICContext, psz: u32) -> Result<u32> {
    let name = read_null_terminated_string_bytes(context, psz)?;
    tracing::warn!("stub MC_uicGetClass({})", EUC_KR.decode(&name).0);
    if name != b"TextComponent" {
        return Ok(0);
    }
    let name = UIC_TEXT_CLASS + size_of::<ComponentClass>() as u32;
    context.write_bytes(name, b"TextComponent\0")?;
    write_generic(
        context,
        UIC_TEXT_CLASS,
        ComponentClass {
            name,
            size: size_of::<TextComponent>() as i32,
            ..ComponentClass::zeroed()
        },
    )?;
    Ok(UIC_TEXT_CLASS)
}

pub async fn create(context: &mut dyn WIPICContext, pac: u32, cls: u32) -> Result<u32> {
    tracing::debug!("MC_uicCreate({pac:#x}, {cls:#x})");
    if cls == 0 {
        return Ok(0);
    }
    if cls != UIC_TEXT_CLASS || pac == 0 {
        return Ok(u32::MAX);
    }
    let screen = read_generic(context, pac)?;
    let cc = context.alloc(size_of::<TextComponent>() as u32)?.0;
    if read_generic::<u32, _>(context, UIC_LAST_DESTROYED)? == cc {
        write_generic(context, UIC_LAST_DESTROYED, 0u32)?;
    }
    let buffer = context.alloc(257)?.0;
    write_generic(context, context.data_ptr(WIPICIndirectPtr(buffer))?, 0u8)?;
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut component = TextComponent {
        class: cls,
        screen,
        maximum: 256,
        buffer,
        background: 0xffffff,
        callback_table: address + offset_of!(TextComponent, callbacks) as u32,
        callback_count: 5,
        ..TextComponent::zeroed()
    };
    for (index, callback) in component.callbacks.iter_mut().enumerate() {
        callback.index = index as u32 + 1;
    }
    write_generic(context, address, component)?;
    Ok(cc)
}

pub async fn destroy(context: &mut dyn WIPICContext, cc: u32) -> Result<()> {
    tracing::debug!("MC_uicDestroy({cc:#x})");
    // A native key handler can destroy the same handle twice before it returns.
    if cc == 0 || read_generic::<u32, _>(context, UIC_LAST_DESTROYED)? == cc {
        return Ok(());
    }
    write_generic(context, UIC_LAST_DESTROYED, cc)?;
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    c.callbacks[0].invoke(context, cc, 0).await?;
    context.free(WIPICIndirectPtr(c.buffer))?;
    context.free(WIPICIndirectPtr(cc))
}

pub async fn configure(context: &mut dyn WIPICContext, cc: u32, x: i32, y: i32, w: i32, h: i32, mask: i32) -> Result<()> {
    tracing::debug!("MC_uicConfigure({cc:#x}, {x}, {y}, {w}, {h}, {mask:#x})");
    if cc == 0 || w <= 0 || h <= 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut component: TextComponent = read_generic(context, address)?;
    if mask & 1 != 0 {
        (component.x, component.y) = (x, y);
    }
    if mask & 2 != 0 {
        (component.width, component.height) = (w, h);
    }
    write_generic(context, address, component)
}

pub async fn get_geometry(context: &mut dyn WIPICContext, cc: u32, px: u32, py: u32, pw: u32, ph: u32) -> Result<()> {
    tracing::debug!("MC_uicGetGeometry({cc:#x}, {px:#x}, {py:#x}, {pw:#x}, {ph:#x})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    for (ptr, value) in [(px, c.x), (py, c.y), (pw, c.width), (ph, c.height)] {
        if ptr != 0 {
            write_generic(context, ptr, value)?;
        }
    }
    Ok(())
}

pub async fn repaint(context: &mut dyn WIPICContext, cc: u32, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
    tracing::debug!("MC_uicRepaint({cc:#x}, {x}, {y}, {w}, {h})");
    if cc == 0 {
        return Ok(());
    }
    context.system().event_queue().push(Event::Repaint);
    Ok(())
}

pub async fn paint(context: &mut dyn WIPICContext, cc: u32, graphics: u32) -> Result<()> {
    paint_with(context, cc, graphics, |context, screen, graphics| {
        let fb = FrameBuffer(read_generic(context, context.data_ptr(screen)?)?);
        let gc: WIPICGraphicsContext = read_generic(context, graphics)?;
        let clip = graphics::context_clip(
            &gc,
            Clip {
                x: 0,
                y: 0,
                width: fb.0.width,
                height: fb.0.height,
            },
        );
        Ok((fb, clip, gc.offset.map(|v| i32::from(v as i16))))
    })
    .await
}

pub async fn paint_with(
    context: &mut dyn WIPICContext,
    cc: u32,
    graphics: u32,
    drawing_target: impl FnOnce(&dyn WIPICContext, WIPICIndirectPtr, u32) -> Result<(FrameBuffer, Clip, [i32; 2])>,
) -> Result<()> {
    tracing::debug!("MC_uicPaint({cc:#x}, {graphics:#x})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    let (fb, clip, offset) = drawing_target(context, WIPICIndirectPtr(c.screen), graphics)?;
    let x = c.x + offset[0];
    let y = c.y + offset[1];
    let rect = Clip {
        x,
        y,
        width: c.width as u32,
        height: c.height as u32,
    };
    let clip = rect.intersect(&clip);
    if clip.width == 0 || clip.height == 0 {
        return Ok(());
    }
    let bytes = c.text(context)?;
    let text = EUC_KR.decode(&bytes).0;
    let font = context.system().platform().font().clone();
    let fg = Rgb8Pixel::to_color(c.foreground);
    let mut canvas = fb.canvas(context)?;
    canvas.fill_rect(x, y, rect.width, rect.height, Rgb8Pixel::to_color(c.background), clip);
    canvas.draw_rect(x, y, rect.width, rect.height, fg, clip);
    let caret = string_width(&font, &EUC_KR.decode(&bytes[..c.cursor as usize]).0, FONT_SIZE) as i32;
    let left = x + 2 - (caret - (c.width - 5).max(0)).max(0);
    let top = y + ((c.height - FONT_SIZE as i32) / 2).max(0);
    canvas.draw_text(&font, &text, left, top, TextAlignment::Left, fg, clip);
    if c.mask & 4 != 0 && c.lost_focus == 0 {
        canvas.draw_line(left + caret, top, left + caret, top + FONT_SIZE as i32, fg, clip);
    }
    canvas.flush()?;
    c.callbacks[1].invoke(context, cc, graphics).await
}

pub async fn edit_text(context: &mut dyn WIPICContext, cc: u32) -> Result<()> {
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let bytes = c.text(context)?;
    if let Some(result) = context.system().platform().text_input(&EUC_KR.decode(&bytes).0).await {
        let replacement = im::encode_input(&result, c.maximum as usize);
        if replacement != bytes {
            c.store(context, cc, &replacement, replacement.len()).await?;
        }
    }
    Ok(())
}

pub async fn handle_event(context: &mut dyn WIPICContext, cc: u32, event_type: i32, param1: i32, param2: i32) -> Result<i32> {
    tracing::debug!("MC_uicHandleEvent({cc:#x}, {event_type}, {param1}, {param2})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let key_press = matches!(event_type, 2 | 502);
    let typing = key_press && matches!(u8::try_from(param1), Ok(b'#' | b'*' | b'0'..=b'9'));
    if c.handler != 0 {
        // A guest handler may translate navigation, repeats or releases into IM key presses.
        let target = if typing && c.mask & 4 != 0 && c.lost_focus == 0 { cc } else { u32::MAX };
        write_generic(context, UIC_INPUT_COMPONENT, target)?;
        let result = context
            .call_function(c.handler, &[cc, event_type as u32, param1 as u32, param2 as u32])
            .await;
        let pending: u32 = read_generic(context, UIC_INPUT_COMPONENT)?;
        write_generic(context, UIC_INPUT_COMPONENT, 0u32)?;
        let consumed = result? != 0;
        if pending == 0 {
            c.callbacks[4].invoke(context, cc, param1 as u32).await?;
            return Ok(1);
        }
        if consumed {
            return Ok(1);
        }
        c = read_generic(context, address)?;
    }
    if matches!(event_type, 0x4008 | 0x4009) {
        c.lost_focus = u32::from(event_type == 0x4009);
        write_generic(context, address, c)?;
        context.system().event_queue().push(Event::Repaint);
        return Ok(1);
    }
    if !key_press || c.mask & 4 == 0 || c.lost_focus != 0 {
        return Ok(0);
    }
    let handled = if typing {
        edit_text(context, cc).await?;
        true
    } else if param1 == -16 && c.cursor > 0 {
        let mut bytes = c.text(context)?;
        let start = boundary(&bytes, c.cursor as usize - 1);
        bytes.drain(start..c.cursor as usize);
        c.store(context, cc, &bytes, start).await?;
        true
    } else {
        false
    };
    c.callbacks[4].invoke(context, cc, param1 as u32).await?;
    Ok(i32::from(handled))
}

pub async fn set_enable(context: &mut dyn WIPICContext, cc: u32, enable: i32) -> Result<()> {
    tracing::debug!("MC_uicSetEnable({cc:#x}, {enable})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    c.mask = (c.mask & !4) | if enable != 0 { 4 } else { 0 };
    write_generic(context, address, c)
}

pub async fn set_callback(context: &mut dyn WIPICContext, cc: u32, index: i32, function: u32, data: u32) -> Result<u32> {
    tracing::debug!("MC_uicSetCallback({cc:#x}, {index}, {function:#x}, {data:#x})");
    if cc == 0 || !(1..=5).contains(&index) {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let callback = &mut c.callbacks[index as usize - 1];
    let previous = replace(&mut callback.function, function);
    callback.data = data;
    write_generic(context, address, c)?;
    Ok(previous)
}

pub async fn set_event_handler(context: &mut dyn WIPICContext, cc: u32, handler: u32) -> Result<u32> {
    tracing::debug!("MC_uicSetEventHandler({cc:#x}, {handler:#x})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let previous = replace(&mut c.handler, handler);
    write_generic(context, address, c)?;
    Ok(previous)
}

pub async fn insert_text(context: &mut dyn WIPICContext, cc: u32, idx: i32, psz: u32, len: i32) -> Result<i32> {
    tracing::debug!("MC_uicInsertText({cc:#x}, {idx}, {psz:#x}, {len})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let mut bytes = c.text(context)?;
    let mut insert = vec![0; len.clamp(0, i32::from(c.maximum - c.size)) as usize];
    context.read_bytes(psz, &mut insert)?;
    let n = boundary(&insert, insert.iter().position(|&b| b == 0).unwrap_or(insert.len()));
    if n != 0 {
        insert.truncate(n);
        let start = boundary(&bytes, idx.max(0) as usize);
        bytes.splice(start..start, insert);
        c.store(context, cc, &bytes, start + n).await?;
    }
    Ok(n as i32)
}

pub async fn delete_text(context: &mut dyn WIPICContext, cc: u32, idx: i32, len: i32) -> Result<()> {
    tracing::debug!("MC_uicDeleteText({cc:#x}, {idx}, {len})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    if !(0..i32::from(c.size)).contains(&idx) || len < -1 {
        return Ok(());
    }
    let mut bytes = c.text(context)?;
    let start = boundary(&bytes, idx as usize);
    let end = if len == -1 {
        bytes.len()
    } else {
        boundary(&bytes, idx as usize + len as usize)
    };
    if end > start {
        bytes.drain(start..end);
        let cursor = c.cursor as usize;
        let cursor = cursor - (cursor.clamp(start, end) - start);
        c.store(context, cc, &bytes, cursor).await?;
    }
    Ok(())
}

pub async fn get_text(context: &mut dyn WIPICContext, cc: u32, idx: i32, destination: u32, len: i32) -> Result<i32> {
    tracing::debug!("MC_uicGetText({cc:#x}, {idx}, {destination:#x}, {len})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    let bytes = c.text(context)?;
    let start = boundary(&bytes, idx.max(0) as usize);
    let n = boundary(&bytes[start..], len.max(0) as usize);
    if n != 0 {
        context.write_bytes(destination, &bytes[start..start + n])?;
    }
    Ok(n as i32)
}

pub async fn set_max_text_size(context: &mut dyn WIPICContext, cc: u32, max: i32) -> Result<i32> {
    tracing::debug!("MC_uicSetMaxTextSize({cc:#x}, {max})");
    if cc == 0 || max < 0 {
        return Ok(-9);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    let old = i32::from(c.maximum);
    if max > i32::from(i16::MAX) {
        return Ok(-17);
    }
    if max != old {
        let bytes = c.text(context)?;
        let n = boundary(&bytes, max as usize);
        let buffer = context.alloc(max as u32 + 1)?;
        write_null_terminated_string_bytes(context, context.data_ptr(buffer)?, &bytes[..n])?;
        context.free(WIPICIndirectPtr(c.buffer))?;
        c.buffer = buffer.0;
        c.maximum = max as i16;
        c.cursor = c.cursor.min(n as i16);
        let changed = c.size != n as i16;
        c.size = n as i16;
        write_generic(context, address, c)?;
        if changed {
            context.system().event_queue().push(Event::Repaint);
            c.callbacks[3].invoke(context, cc, 0).await?;
        }
    }
    Ok(old)
}

pub async fn get_text_size(context: &mut dyn WIPICContext, cc: u32) -> Result<i32> {
    tracing::debug!("MC_uicGetTextSize({cc:#x})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    Ok(i32::from(c.size))
}

pub async fn get_max_text_size(context: &mut dyn WIPICContext, cc: u32) -> Result<i32> {
    tracing::debug!("MC_uicGetMaxTextSize({cc:#x})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    Ok(i32::from(c.maximum))
}

pub async fn get_cursor_position(context: &mut dyn WIPICContext, cc: u32) -> Result<i32> {
    tracing::debug!("OEMC_uicGetCursorPosition({cc:#x})");
    if cc == 0 {
        return Ok(-9);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let c: TextComponent = read_generic(context, address)?;
    Ok(i32::from(c.cursor))
}

pub async fn get_font(context: &mut dyn WIPICContext, cc: u32) -> Result<i32> {
    tracing::debug!("MC_uicGetFont({cc:#x})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    read_generic(context, address + offset_of!(TextComponent, font) as u32)
}

pub async fn set_fg_color(context: &mut dyn WIPICContext, cc: u32, color: u32) -> Result<()> {
    tracing::debug!("MC_uicSetFgColor({cc:#x}, {color:#x})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    write_generic(context, address + offset_of!(TextComponent, foreground) as u32, color)
}

pub async fn set_bg_color(context: &mut dyn WIPICContext, cc: u32, color: u32) -> Result<()> {
    tracing::debug!("MC_uicSetBgColor({cc:#x}, {color:#x})");
    if cc == 0 {
        return Ok(());
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    write_generic(context, address + offset_of!(TextComponent, background) as u32, color)
}

pub async fn set_font(context: &mut dyn WIPICContext, cc: u32, font: i32) -> Result<i32> {
    tracing::debug!("MC_uicSetFont({cc:#x}, {font})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let addr = address + offset_of!(TextComponent, font) as u32;
    let previous = read_generic(context, addr)?;
    write_generic(context, addr, font)?;
    Ok(previous)
}

pub async fn set_cursor_position(context: &mut dyn WIPICContext, cc: u32, idx: i32) -> Result<i32> {
    tracing::debug!("OEMC_uicSetCursorPosition({cc:#x}, {idx})");
    if cc == 0 {
        return Ok(-9);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let mut c: TextComponent = read_generic(context, address)?;
    if idx < 0 || boundary(&c.text(context)?, idx as usize) != idx as usize {
        return Ok(-9);
    }
    c.cursor = idx as i16;
    write_generic(context, address, c)?;
    Ok(0)
}

pub async fn get_class_name(context: &mut dyn WIPICContext, cc: u32) -> Result<u32> {
    tracing::debug!("MC_uicGetClassName({cc:#x})");
    if cc == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let class: u32 = read_generic(context, address)?;
    read_generic(context, class + offset_of!(ComponentClass, name) as u32)
}

pub async fn is_instance(context: &mut dyn WIPICContext, cc: u32, name: u32) -> Result<u32> {
    tracing::debug!("MC_uicIsInstance({cc:#x}, {name:#x})");
    if cc == 0 || name == 0 {
        return Ok(0);
    }
    let address = context.data_ptr(WIPICIndirectPtr(cc))?;
    let class: u32 = read_generic(context, address)?;
    Ok(u32::from(
        class == UIC_TEXT_CLASS && read_null_terminated_string_bytes(context, name)? == b"TextComponent",
    ))
}

pub async fn set_label_alignment(_context: &mut dyn WIPICContext, cc: u32, align: i32) -> Result<i32> {
    tracing::warn!("stub MC_uicSetLabelAlignment({cc:#x}, {align})");
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
    use super::*;
    use crate::context::test::TestContext;
    use alloc::boxed::Box;
    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};

    #[futures_test::test]
    async fn text_field_edits_and_paints_guest_text() -> Result<()> {
        let platform = TestPlatform::new().with_text_input("가나");
        let system = System::new(Box::new(platform), "test-pid", "test-aid", DefaultTaskRunner);
        system.platform().screen().resize(32, 32)?;
        let mut ctx = TestContext::with_system(system);
        let buf = ctx.alloc_raw(16)?;
        write_null_terminated_string_bytes(&mut ctx, buf, b"TextComponent")?;
        let app = create_application_context(&mut ctx).await?;
        let class = get_class(&mut ctx, buf).await?;
        let cc = create(&mut ctx, app, class).await?;
        configure(&mut ctx, cc, 2, 3, 24, 16, 3).await?;
        set_enable(&mut ctx, cc, 1).await?;
        set_max_text_size(&mut ctx, cc, 4).await?;
        write_generic(&mut ctx, buf, b'A')?;
        insert_text(&mut ctx, cc, 0, buf, 1).await?;
        for (key, expected) in [(53, b"A\xb0\xa1".as_slice()), (-16, b"A")] {
            handle_event(&mut ctx, cc, 2, key, 0).await?;
            let len = get_text(&mut ctx, cc, 0, buf, 4).await?;
            let bytes: [u8; 4] = read_generic(&ctx, buf)?;
            assert_eq!(&bytes[..len as usize], expected);
        }
        let gc = ctx.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
        graphics::init_context(&mut ctx, gc).await?;
        set_fg_color(&mut ctx, cc, 0xff0000).await?;
        paint(&mut ctx, cc, gc).await?;
        let screen = WIPICIndirectPtr(read_generic(&ctx, app)?);
        graphics::get_rgb_pixels(&mut ctx, screen, 2, 3, 1, 1, buf, 4).await?;
        assert_eq!(read_generic::<u32, _>(&ctx, buf)?, 0xff0000);
        destroy(&mut ctx, cc).await?;
        destroy(&mut ctx, cc).await
    }
}
