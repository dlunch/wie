use alloc::vec;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wipi_types::{
    lgt::wipic::{LgtFramebuffer, LgtGraphicsContext, LgtGraphicsView, LgtImage},
    wipic::{WIPICDisplayInfo, WIPICFramebuffer, WIPICIndirectPtr, WIPICWord},
};

use wie_backend::canvas::{ArgbPixel, Clip, Color, Image, PixelType, VecImageBuffer};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, WieError, read_generic, write_generic};
use wie_wipi_c::{
    WIPICContext,
    api::graphics::{FrameBuffer, decode_image_framebuffer, primitives},
};

const GRAPHICS_STATE_ROOT: u32 = 0x7fff1008;
const DISPLAY_PROPERTIES_ROOT: u32 = 0x7fff1010;
const FRAMEBUFFER_DEPTH: u32 = 16;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct LgtGraphicsState {
    physical_width: u32,
    physical_height: u32,
    use_annunciator: u32,
    application_view_active: u32,
    ptr_screen_backing: u32,
    ptr_screen_view: u32,
    ptr_annunciator_view: u32,
    ptr_screen_wrapper: u32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct LgtDisplayProperties {
    physical_width: u32,
    physical_height: u32,
    use_annunciator: u32,
}

// These records are read directly by the LGT native graphics code. Keep the
// guest ABI sizes explicit so an accidental Rust layout change fails at build time.
const LGT_FRAMEBUFFER_SIZE: usize = 16;
const LGT_GRAPHICS_VIEW_SIZE: usize = 20;
const LGT_IMAGE_SIZE: usize = 8;
const LGT_GRAPHICS_CONTEXT_SIZE: usize = 56;

const _: () = {
    assert!(size_of::<LgtFramebuffer>() == LGT_FRAMEBUFFER_SIZE);
    assert!(size_of::<LgtGraphicsView>() == LGT_GRAPHICS_VIEW_SIZE);
    assert!(size_of::<LgtImage>() == LGT_IMAGE_SIZE);
    assert!(size_of::<LgtGraphicsContext>() == LGT_GRAPHICS_CONTEXT_SIZE);
};

struct ResolvedFramebuffer {
    framebuffer: FrameBuffer,
    public: LgtFramebuffer,
    view: LgtGraphicsView,
}

struct ResolvedContext {
    clip: Clip,
    translation_x: i32,
    translation_y: i32,
    foreground: u32,
}

pub fn init_process_state(core: &mut ArmCore, physical_width: u32, physical_height: u32) -> Result<()> {
    let ptr_state: u32 = read_generic(core, GRAPHICS_STATE_ROOT)?;
    if ptr_state != 0 {
        return Ok(());
    }

    let properties: LgtDisplayProperties = read_generic(core, DISPLAY_PROPERTIES_ROOT)?;
    let state = LgtGraphicsState {
        physical_width: if properties.physical_width == 0 {
            physical_width
        } else {
            properties.physical_width
        },
        physical_height: if properties.physical_height == 0 {
            physical_height
        } else {
            properties.physical_height
        },
        use_annunciator: properties.use_annunciator,
        application_view_active: 0,
        ptr_screen_backing: 0,
        ptr_screen_view: 0,
        ptr_annunciator_view: 0,
        ptr_screen_wrapper: 0,
    };
    let ptr_state = Allocator::alloc(core, size_of::<LgtGraphicsState>() as u32)?;
    write_generic(core, ptr_state, state)?;
    write_generic(core, GRAPHICS_STATE_ROOT, ptr_state)
}

#[derive(Clone, Copy)]
#[repr(u32)]
enum DisplayProperty {
    Width = 0x64,
    Height = 0x65,
    UseAnnunciator = 0x7f,
}

impl TryFrom<u32> for DisplayProperty {
    type Error = ();

    fn try_from(value: u32) -> core::result::Result<Self, Self::Error> {
        match value {
            0x64 => Ok(Self::Width),
            0x65 => Ok(Self::Height),
            0x7f => Ok(Self::UseAnnunciator),
            _ => Err(()),
        }
    }
}

pub async fn set_display_property(core: &mut ArmCore, _: &mut (), _ptr_display: u32, property: u32, value: u32, size: u32) -> Result<u32> {
    if size != 0 {
        return Ok(0);
    }
    let Ok(property) = DisplayProperty::try_from(property) else {
        return Ok(0);
    };

    let mut properties: LgtDisplayProperties = read_generic(core, DISPLAY_PROPERTIES_ROOT)?;

    match property {
        DisplayProperty::Width => properties.physical_width = value,
        DisplayProperty::Height => properties.physical_height = value,
        DisplayProperty::UseAnnunciator => properties.use_annunciator = value,
    }
    write_generic(core, DISPLAY_PROPERTIES_ROOT, properties)?;

    let ptr_state: u32 = read_generic(core, GRAPHICS_STATE_ROOT)?;
    if ptr_state != 0 {
        let mut state: LgtGraphicsState = read_generic(core, ptr_state)?;
        match property {
            DisplayProperty::Width => state.physical_width = value,
            DisplayProperty::Height => state.physical_height = value,
            DisplayProperty::UseAnnunciator => state.use_annunciator = value,
        }
        write_generic(core, ptr_state, state)?;
    }

    Ok(0)
}

pub fn set_use_annunciator(core: &mut ArmCore, value: u32) -> Result<()> {
    let ptr_state: u32 = read_generic(core, GRAPHICS_STATE_ROOT)?;
    let mut state: LgtGraphicsState = read_generic(core, ptr_state)?;
    state.use_annunciator = value;
    write_generic(core, ptr_state, state)
}

fn state(context: &dyn WIPICContext) -> Result<(u32, LgtGraphicsState)> {
    let ptr_state: u32 = read_generic(context, GRAPHICS_STATE_ROOT)?;
    if ptr_state == 0 {
        return Err(WieError::FatalError("LGT graphics state is not initialized".into()));
    }
    Ok((ptr_state, read_generic(context, ptr_state)?))
}

fn annunciator_height(width: u32) -> u32 {
    match width {
        120 => 14,
        176 | 220 => 20,
        240 | 320 | 400 => 24,
        _ => 0,
    }
}

fn requested_application_y(state: &LgtGraphicsState) -> u32 {
    if state.use_annunciator != 0 {
        annunciator_height(state.physical_width)
    } else {
        0
    }
}

fn application_y(state: &LgtGraphicsState) -> u32 {
    if state.application_view_active != 0 {
        requested_application_y(state)
    } else {
        0
    }
}

fn resize_presentation(context: &mut dyn WIPICContext, width: u32, height: u32) -> Result<()> {
    let screen = context.system().platform().screen();
    if screen.width() != width || screen.height() != height {
        screen.resize(width, height)?;
    }
    Ok(())
}

fn presentation_image(image: &dyn Image, view: LgtGraphicsView, width: u32, height: u32) -> Result<VecImageBuffer<ArgbPixel>> {
    if view.x < 0
        || view.y < 0
        || i64::from(view.x) + i64::from(view.width) > i64::from(image.width())
        || i64::from(view.y) + i64::from(view.height) > i64::from(image.height())
        || view.width > width
        || view.height > height
    {
        return Err(WieError::FatalError("LGT presentation view is outside its physical backing".into()));
    }

    let pixel_count = width.checked_mul(height).ok_or(WieError::AllocationFailure)? as usize;
    let mut pixels = vec![ArgbPixel::from_color(Color { a: 0xff, r: 0, g: 0, b: 0 }); pixel_count];
    let destination_x = (width - view.width) / 2;
    let destination_y = (height - view.height) / 2;
    for y in 0..view.height {
        for x in 0..view.width {
            let destination = ((destination_y + y) * width + destination_x + x) as usize;
            pixels[destination] = ArgbPixel::from_color(image.get_pixel(view.x + x as i32, view.y + y as i32));
        }
    }
    Ok(VecImageBuffer::<ArgbPixel>::from_raw(width, height, pixels))
}

fn alloc_record<T: Pod>(context: &mut dyn WIPICContext, value: T) -> Result<WIPICIndirectPtr> {
    let handle = context.alloc(size_of::<T>() as u32)?;
    let address = context.data_ptr(handle)?;
    write_generic(context, address, value)?;
    Ok(handle)
}

fn read_record<T: Pod>(context: &dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<T> {
    let address = context.data_ptr(handle)?;
    read_generic(context, address)
}

fn create_backing(context: &mut dyn WIPICContext, width: u32, height: u32) -> Result<WIPICIndirectPtr> {
    let bpl = width.checked_mul(2).ok_or(WieError::AllocationFailure)?;
    let size = bpl.checked_mul(height).ok_or(WieError::AllocationFailure)?;
    let pixels = context.alloc(size)?;
    let pixels_address = context.data_ptr(pixels)?;
    context.write_bytes(pixels_address, &vec![0; size as usize])?;

    alloc_record(
        context,
        WIPICFramebuffer {
            width,
            height,
            bpl,
            bpp: FRAMEBUFFER_DEPTH,
            buf: pixels,
        },
    )
}

fn create_view(context: &mut dyn WIPICContext, ptr_backing: WIPICIndirectPtr, x: i32, y: i32, width: u32, height: u32) -> Result<WIPICIndirectPtr> {
    alloc_record(
        context,
        LgtGraphicsView {
            ptr_backing: ptr_backing.0,
            x,
            y,
            width,
            height,
        },
    )
}

fn resolve_framebuffer(context: &dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<ResolvedFramebuffer> {
    let public: LgtFramebuffer = read_record(context, handle)?;
    if public.ptr_graphics == 0 {
        return Err(WieError::FatalError(alloc::format!(
            "LGT framebuffer {:#x} has no graphics view",
            handle.0
        )));
    }
    let view: LgtGraphicsView = read_record(context, WIPICIndirectPtr(public.ptr_graphics))?;
    if view.ptr_backing == 0 {
        return Err(WieError::FatalError(alloc::format!(
            "LGT graphics view {:#x} has no backing",
            public.ptr_graphics
        )));
    }
    let backing: WIPICFramebuffer = read_record(context, WIPICIndirectPtr(view.ptr_backing))?;

    Ok(ResolvedFramebuffer {
        framebuffer: FrameBuffer(backing),
        public,
        view,
    })
}

fn resolve_context(context: &dyn WIPICContext, framebuffer: &ResolvedFramebuffer, ptr_context: WIPICWord) -> Result<ResolvedContext> {
    let raw: LgtGraphicsContext = read_generic(context, ptr_context)?;
    let (_, state) = state(context)?;
    Ok(normalize_context(raw, framebuffer.public, framebuffer.view, state))
}

fn normalize_context(raw: LgtGraphicsContext, framebuffer: LgtFramebuffer, view: LgtGraphicsView, state: LgtGraphicsState) -> ResolvedContext {
    let adjust_y = if framebuffer.owned_image == 0 && framebuffer.screen_kind != 1 {
        application_y(&state) as i32
    } else {
        0
    };
    let width = (i64::from(raw.clip_x2) + 1 - i64::from(raw.clip_x1)).clamp(0, u32::MAX as i64) as u32;
    let height = (i64::from(raw.clip_y2) + 1 - i64::from(raw.clip_y1)).clamp(0, u32::MAX as i64) as u32;
    let context_clip = Clip {
        x: raw.clip_x1,
        y: raw.clip_y1.wrapping_add(adjust_y),
        width,
        height,
    };
    let view_clip = Clip {
        x: view.x,
        y: view.y,
        width: view.width,
        height: view.height,
    };

    ResolvedContext {
        clip: context_clip.intersect(&view_clip),
        translation_x: raw.offset_x,
        translation_y: raw.offset_y.wrapping_add(adjust_y),
        foreground: raw.foreground,
    }
}

fn image_framebuffer(context: &dyn WIPICContext, ptr_image: WIPICIndirectPtr) -> Result<FrameBuffer> {
    let image: LgtImage = read_record(context, ptr_image)?;
    if image.ptr_image == 0 {
        return Err(WieError::FatalError("LGT image has no native backing".into()));
    }
    Ok(FrameBuffer(read_record(context, WIPICIndirectPtr(image.ptr_image))?))
}

fn activate_application_view(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<()> {
    let (ptr_state, mut state) = state(context)?;
    if state.application_view_active != 0 || state.use_annunciator == 0 || state.ptr_screen_wrapper != handle.0 {
        return Ok(());
    }

    let app_y = requested_application_y(&state);
    let app_height = state.physical_height.checked_sub(app_y).ok_or_else(|| {
        WieError::FatalError(alloc::format!(
            "Invalid LGT display region: {}x{} with annunciator {app_y}",
            state.physical_width,
            state.physical_height
        ))
    })?;
    let ptr_screen_view_handle = WIPICIndirectPtr(state.ptr_screen_view);
    let mut screen_view: LgtGraphicsView = read_record(context, ptr_screen_view_handle)?;
    screen_view.y = app_y as i32;
    screen_view.height = app_height;
    write_generic(context, context.data_ptr(ptr_screen_view_handle)?, screen_view)?;

    let ptr_annunciator_view = create_view(context, WIPICIndirectPtr(state.ptr_screen_backing), 0, 0, state.physical_width, app_y)?;
    state.application_view_active = 1;
    state.ptr_annunciator_view = ptr_annunciator_view.0;
    write_generic(context, ptr_state, state)
}

pub async fn get_screen_framebuffer(context: &mut dyn WIPICContext, kind: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({kind:#x})");
    if kind != 0 {
        return Err(WieError::FatalError(alloc::format!("Unsupported LGT screen kind {kind}")));
    }

    let (ptr_state, mut state) = state(context)?;
    if state.ptr_screen_wrapper != 0 {
        return Ok(WIPICIndirectPtr(state.ptr_screen_wrapper));
    }

    let ptr_backing = create_backing(context, state.physical_width, state.physical_height)?;
    let ptr_screen_view = create_view(context, ptr_backing, 0, 0, state.physical_width, state.physical_height)?;
    let ptr_wrapper = alloc_record(
        context,
        LgtFramebuffer {
            owned_image: 0,
            ptr_graphics: ptr_screen_view.0,
            ptr_image: 0,
            screen_kind: kind,
        },
    )?;

    state.ptr_screen_backing = ptr_backing.0;
    state.ptr_screen_view = ptr_screen_view.0;
    state.ptr_annunciator_view = 0;
    state.ptr_screen_wrapper = ptr_wrapper.0;
    write_generic(context, ptr_state, state)?;
    resize_presentation(context, state.physical_width, state.physical_height)?;

    Ok(ptr_wrapper)
}

pub async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, width: i32, height: i32) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({width}, {height})");
    if width <= 0 || height <= 0 {
        return Err(WieError::AllocationFailure);
    }

    let ptr_backing = create_backing(context, width as u32, height as u32)?;
    let ptr_view = create_view(context, ptr_backing, 0, 0, width as u32, height as u32)?;
    alloc_record(
        context,
        LgtFramebuffer {
            owned_image: 1,
            ptr_graphics: ptr_view.0,
            ptr_image: ptr_backing.0,
            screen_kind: 0,
        },
    )
}

pub async fn destroy_offscreen_framebuffer(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyOffScreenFrameBuffer({:#x})", handle.0);
    let public: LgtFramebuffer = read_record(context, handle)?;
    if public.owned_image == 0 || public.ptr_graphics == 0 || public.ptr_image == 0 {
        return Err(WieError::FatalError(alloc::format!("Invalid LGT off-screen framebuffer {:#x}", handle.0)));
    }
    let backing: WIPICFramebuffer = read_record(context, WIPICIndirectPtr(public.ptr_image))?;

    context.free(WIPICIndirectPtr(public.ptr_graphics))?;
    context.free(backing.buf)?;
    context.free(WIPICIndirectPtr(public.ptr_image))?;
    context.free(handle)
}

pub async fn get_framebuffer_pointer(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<WIPICWord> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_POINTER({:#x})", handle.0);
    activate_application_view(context, handle)?;
    Ok(resolve_framebuffer(context, handle)?.framebuffer.0.buf.0)
}

pub async fn get_framebuffer_width(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_WIDTH({:#x})", handle.0);
    Ok(resolve_framebuffer(context, handle)?.view.width as i32)
}

pub async fn get_framebuffer_height(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_HEIGHT({:#x})", handle.0);
    Ok(resolve_framebuffer(context, handle)?.view.height as i32)
}

pub async fn get_framebuffer_bpl(context: &mut dyn WIPICContext, handle: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPL({:#x})", handle.0);
    let resolved = resolve_framebuffer(context, handle)?;
    if resolved.public.owned_image != 0 {
        Ok(0)
    } else {
        Ok(resolved.framebuffer.0.bpl as i32)
    }
}

pub async fn get_framebuffer_bpp(context: &mut dyn WIPICContext, _handle: WIPICIndirectPtr) -> Result<i32> {
    // The native accessor ignores its argument and reads display property 0x6e,
    // property 2.  The application consequently passes a non-framebuffer value
    // here during screen initialization.
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPP() -> {FRAMEBUFFER_DEPTH}");
    let (_, state) = state(context)?;
    if state.physical_width == 0 || state.physical_height == 0 {
        return Err(WieError::FatalError("Invalid LGT display dimensions".into()));
    }
    Ok(FRAMEBUFFER_DEPTH as i32)
}

pub async fn init_context(context: &mut dyn WIPICContext, ptr_context: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpInitContext({ptr_context:#x})");
    write_generic(
        context,
        ptr_context,
        LgtGraphicsContext {
            clip_x1: 0,
            clip_y1: 0,
            clip_x2: 0x7fff,
            clip_y2: 0x7fff,
            foreground: 0,
            background: 0x00ff_ffff,
            alpha: 255,
            pixel_op: 0,
            pixel_param: 255,
            font: 0,
            style: 0,
            xor_enabled: 0,
            offset_x: 0,
            offset_y: 0,
        },
    )
}

pub async fn set_context(context: &mut dyn WIPICContext, ptr_context: WIPICWord, operation: WIPICWord, value: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpSetContext({ptr_context:#x}, {operation}, {value:#x})");
    let mut graphics: LgtGraphicsContext = read_generic(context, ptr_context)?;
    match operation {
        0 if value != 0 => {
            graphics.clip_x1 = read_generic(context, value)?;
            graphics.clip_y1 = read_generic(context, value + 4)?;
            graphics.clip_x2 = read_generic::<i32, _>(context, value + 8)?.wrapping_sub(1);
            graphics.clip_y2 = read_generic::<i32, _>(context, value + 12)?.wrapping_sub(1);
        }
        1 => graphics.foreground = value,
        2 => graphics.background = value,
        3 => {}
        4 if value <= 255 => graphics.alpha = value,
        5 => graphics.pixel_op = value,
        6 => graphics.pixel_param = value,
        7 => graphics.font = value,
        8 => graphics.style = value,
        9 if value == 0 => {
            graphics.pixel_op = 0;
            graphics.xor_enabled = 0;
        }
        9 => {
            graphics.alpha = 0;
            graphics.pixel_op = 1;
            graphics.xor_enabled = 1;
        }
        10 if value != 0 => {
            graphics.offset_x = read_generic(context, value)?;
            graphics.offset_y = read_generic(context, value + 4)?;
        }
        _ => {}
    }
    write_generic(context, ptr_context, graphics)
}

pub async fn put_pixel(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x: i32, y: i32, ptr_graphics: WIPICWord) -> Result<()> {
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::put_pixel(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        color,
        graphics.clip,
    )
}

pub async fn fill_rect(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::fill_rect(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width as u32,
        height as u32,
        color,
        graphics.clip,
    )
}

pub async fn draw_line(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::draw_line(
        context,
        &resolved.framebuffer,
        x1.wrapping_add(graphics.translation_x),
        y1.wrapping_add(graphics.translation_y),
        x2.wrapping_add(graphics.translation_x),
        y2.wrapping_add(graphics.translation_y),
        color,
        graphics.clip,
    )
}

pub async fn draw_rect(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::draw_rect(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width.saturating_sub(1) as u32,
        height.saturating_sub(1) as u32,
        color,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    start_angle: i32,
    end_angle: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::draw_arc(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width.saturating_sub(1) as u32,
        height.saturating_sub(1) as u32,
        start_angle,
        end_angle.wrapping_sub(start_angle),
        color,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn fill_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    start_angle: i32,
    end_angle: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::fill_arc(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width as u32,
        height as u32,
        start_angle,
        end_angle.wrapping_sub(start_angle),
        color,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_image(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    ptr_image: WIPICIndirectPtr,
    source_x: i32,
    source_y: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let source = image_framebuffer(context, ptr_image)?.image(context)?;
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    primitives::draw_image(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width as u32,
        height as u32,
        &*source,
        source_x,
        source_y,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_frame_buffer(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    source: WIPICIndirectPtr,
    source_x: i32,
    source_y: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let source = resolve_framebuffer(context, source)?.framebuffer.image(context)?;
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    primitives::draw_image(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width as u32,
        height as u32,
        &*source,
        source_x,
        source_y,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    source_x: i32,
    source_y: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let source = resolved.framebuffer.image(context)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    primitives::draw_image(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width as u32,
        height as u32,
        &*source,
        source_x.wrapping_add(graphics.translation_x),
        source_y.wrapping_add(graphics.translation_y),
        graphics.clip,
    )
}

pub async fn draw_string(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    ptr_string: WIPICWord,
    length: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(());
    };
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    let color = resolved.framebuffer.pixel_to_color(graphics.foreground);
    primitives::draw_text(
        context,
        &resolved.framebuffer,
        &string,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        color,
        graphics.clip,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    source: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    destination: WIPICWord,
    destination_bpl: i32,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let image = resolve_framebuffer(context, source)?.framebuffer.image(context)?;
    primitives::get_rgb_pixels(context, &*image, x, y, width, height, destination, destination_bpl)
}

#[allow(clippy::too_many_arguments)]
pub async fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    source: WIPICWord,
    source_bpl: i32,
    ptr_graphics: WIPICWord,
) -> Result<()> {
    if width <= 0 || height <= 0 {
        return Ok(());
    }
    let resolved = resolve_framebuffer(context, dst)?;
    let graphics = resolve_context(context, &resolved, ptr_graphics)?;
    primitives::set_rgb_pixels(
        context,
        &resolved.framebuffer,
        x.wrapping_add(graphics.translation_x),
        y.wrapping_add(graphics.translation_y),
        width,
        height,
        source,
        source_bpl,
        graphics.clip,
    )
}

pub async fn flush_lcd(
    context: &mut dyn WIPICContext,
    kind: WIPICWord,
    source: WIPICIndirectPtr,
    _x: WIPICWord,
    _y: WIPICWord,
    _width: WIPICWord,
    _height: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFlushLcd({kind}, {:#x})", source.0);
    if kind != 0 {
        return Err(WieError::FatalError(alloc::format!("Unsupported LGT display kind {kind}")));
    }
    let resolved = resolve_framebuffer(context, source)?;
    let image = resolved.framebuffer.image(context)?;
    let (_, state) = state(context)?;
    let presented = presentation_image(&*image, resolved.view, state.physical_width, state.physical_height)?;
    resize_presentation(context, state.physical_width, state.physical_height)?;
    context.system().platform().screen().paint(&presented);
    Ok(())
}

pub async fn get_display_info(context: &mut dyn WIPICContext, kind: WIPICWord, output: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetDisplayInfo({kind}, {output:#x})");
    if kind != 0 {
        return Err(WieError::FatalError(alloc::format!("Unsupported LGT display kind {kind}")));
    }
    let (_, state) = state(context)?;
    let height = state
        .physical_height
        .checked_sub(application_y(&state))
        .ok_or_else(|| WieError::FatalError("Invalid LGT logical display height".into()))?;
    write_generic(
        context,
        output,
        WIPICDisplayInfo {
            bpp: FRAMEBUFFER_DEPTH,
            depth: FRAMEBUFFER_DEPTH,
            width: state.physical_width,
            height,
            bpl: state.physical_width * 2,
            color_type: 1,
            red_mask: 0xf800,
            green_mask: 0x07e0,
            blue_mask: 0x001f,
        },
    )?;
    Ok(1)
}

pub async fn create_image(
    context: &mut dyn WIPICContext,
    output: WIPICWord,
    encoded_data: WIPICIndirectPtr,
    offset: WIPICWord,
    length: WIPICWord,
) -> Result<WIPICWord> {
    let backing = decode_image_framebuffer(context, encoded_data, offset, length)?;
    let width = backing.0.width;
    let height = backing.0.height;
    let ptr_backing = alloc_record(context, backing.0)?;
    let ptr_view = create_view(context, ptr_backing, 0, 0, width, height)?;
    let ptr_framebuffer = alloc_record(
        context,
        LgtFramebuffer {
            owned_image: 1,
            ptr_graphics: ptr_view.0,
            ptr_image: ptr_backing.0,
            screen_kind: 0,
        },
    )?;
    let public = alloc_record(
        context,
        LgtImage {
            ptr_image: ptr_backing.0,
            ptr_framebuffer: ptr_framebuffer.0,
        },
    )?;
    write_generic(context, output, public)?;
    Ok(1)
}

pub async fn get_image_framebuffer(context: &mut dyn WIPICContext, ptr_image: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    Ok(WIPICIndirectPtr(read_record::<LgtImage>(context, ptr_image)?.ptr_framebuffer))
}

pub async fn get_image_property(context: &mut dyn WIPICContext, ptr_image: WIPICIndirectPtr, property: i32) -> Result<i32> {
    let backing = image_framebuffer(context, ptr_image)?;
    Ok(match property {
        4 => backing.0.width as i32,
        5 => backing.0.height as i32,
        _ => 0,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use futures::FutureExt;
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, read_generic};

    use super::{
        DISPLAY_PROPERTIES_ROOT, GRAPHICS_STATE_ROOT, LgtDisplayProperties, LgtFramebuffer, LgtGraphicsContext, LgtGraphicsState, LgtGraphicsView,
        application_y, init_process_state, normalize_context, presentation_image, set_display_property,
    };
    use wie_backend::canvas::{ArgbPixel, Image, PixelType, VecImageBuffer};

    #[test]
    fn process_state_initializes_once_for_the_clet_lifecycle() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        init_process_state(&mut core, 240, 320)?;
        let ptr_state: u32 = read_generic(&core, GRAPHICS_STATE_ROOT)?;
        let state: LgtGraphicsState = read_generic(&core, ptr_state)?;
        assert_eq!(state.physical_width, 240);
        assert_eq!(state.physical_height, 320);

        init_process_state(&mut core, 176, 220)?;
        let ptr_state_after: u32 = read_generic(&core, GRAPHICS_STATE_ROOT)?;
        let state_after: LgtGraphicsState = read_generic(&core, ptr_state_after)?;
        assert_eq!(ptr_state_after, ptr_state);
        assert_eq!(state_after.physical_width, 240);
        assert_eq!(state_after.physical_height, 320);

        Ok(())
    }

    #[test]
    fn display_properties_update_physical_display_state() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        set_display_property(&mut core, &mut (), 0, 0x64, 240, 0).now_or_never().unwrap()?;
        set_display_property(&mut core, &mut (), 0, 0x65, 320, 0).now_or_never().unwrap()?;
        let properties: LgtDisplayProperties = read_generic(&core, DISPLAY_PROPERTIES_ROOT)?;
        assert_eq!(properties.physical_width, 240);
        init_process_state(&mut core, 176, 220)?;

        let ptr_state: u32 = read_generic(&core, GRAPHICS_STATE_ROOT)?;
        let state: LgtGraphicsState = read_generic(&core, ptr_state)?;
        assert_eq!(state.physical_width, 240);
        assert_eq!(state.physical_height, 320);

        Ok(())
    }

    #[test]
    fn presentation_centers_the_application_view_in_the_physical_display() {
        let source = VecImageBuffer::<ArgbPixel>::from_raw(2, 3, vec![0xff000001, 0xff000002, 0xff000003, 0xff000004, 0xff000005, 0xff000006]);
        let view = LgtGraphicsView {
            ptr_backing: 0,
            x: 0,
            y: 1,
            width: 2,
            height: 2,
        };

        let presented = presentation_image(&source, view, 2, 4).unwrap();

        assert_eq!(presented.width(), 2);
        assert_eq!(presented.height(), 4);
        assert_eq!(ArgbPixel::from_color(presented.get_pixel(0, 0)), 0xff000000);
        assert_eq!(ArgbPixel::from_color(presented.get_pixel(0, 1)), 0xff000003);
        assert_eq!(ArgbPixel::from_color(presented.get_pixel(1, 2)), 0xff000006);
        assert_eq!(ArgbPixel::from_color(presented.get_pixel(1, 3)), 0xff000000);
    }

    #[test]
    fn annunciator_request_only_changes_application_geometry_after_raw_screen_access() {
        let mut state = LgtGraphicsState {
            physical_width: 240,
            physical_height: 320,
            use_annunciator: 1,
            application_view_active: 0,
            ptr_screen_backing: 0,
            ptr_screen_view: 0,
            ptr_annunciator_view: 0,
            ptr_screen_wrapper: 0,
        };

        assert_eq!(application_y(&state), 0);
        state.application_view_active = 1;
        assert_eq!(application_y(&state), 24);
        state.use_annunciator = 0;
        assert_eq!(application_y(&state), 0);
    }

    #[test]
    fn context_normalization_applies_lgt_view_and_annunciator_coordinates() {
        let state = LgtGraphicsState {
            physical_width: 240,
            physical_height: 320,
            use_annunciator: 1,
            application_view_active: 1,
            ptr_screen_backing: 0,
            ptr_screen_view: 0,
            ptr_annunciator_view: 0,
            ptr_screen_wrapper: 0,
        };
        let framebuffer = LgtFramebuffer {
            owned_image: 0,
            ptr_graphics: 0,
            ptr_image: 0,
            screen_kind: 0,
        };
        let view = LgtGraphicsView {
            ptr_backing: 0,
            x: 0,
            y: 24,
            width: 240,
            height: 296,
        };
        let raw = LgtGraphicsContext {
            clip_x1: 10,
            clip_y1: 2,
            clip_x2: 29,
            clip_y2: 11,
            foreground: 0xf800,
            background: 0,
            alpha: 255,
            pixel_op: 0,
            pixel_param: 0,
            font: 0,
            style: 0,
            xor_enabled: 0,
            offset_x: 3,
            offset_y: 4,
        };

        let normalized = normalize_context(raw, framebuffer, view, state);

        assert_eq!(normalized.clip.x, 10);
        assert_eq!(normalized.clip.y, 26);
        assert_eq!(normalized.clip.width, 20);
        assert_eq!(normalized.clip.height, 10);
        assert_eq!(normalized.translation_x, 3);
        assert_eq!(normalized.translation_y, 28);
        assert_eq!(normalized.foreground, 0xf800);
    }
}
