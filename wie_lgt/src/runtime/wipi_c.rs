use alloc::{boxed::Box, format, string::ToString, vec};

mod context;

use bytemuck::{Pod, Zeroable};

use jvm::{Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_rust::ClassDefinitionImpl;

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, write_null_terminated_string_bytes};
use wie_wipi_c::{
    MethodImpl, WIPICContext,
    api::{database, graphics, kernel, media, misc, net},
};

use context::LgtWIPICContext;

use crate::runtime::java::classes::net::wie::{CletWrapper, CletWrapperCard, CletWrapperContext};

pub fn get_wipi_c_method(core: &mut ArmCore, system: &mut System, jvm: &Jvm, function_index: u32) -> Result<u32> {
    let method = match function_index {
        0x03 => return core.register_function(clet_register, jvm),
        0x32 => graphics::get_framebuffer_pointer.into_body(),
        0x33 => graphics::get_framebuffer_width.into_body(),
        0x34 => graphics::get_framebuffer_height.into_body(),
        0x36 => graphics::get_framebuffer_bpp.into_body(),
        0x64 => kernel::printk.into_body(),
        0x65 => kernel::sprintk.into_body(),
        0x6a => unk1.into_body(),
        0x75 => kernel::alloc.into_body(),
        0x76 => kernel::calloc.into_body(),
        0x77 => kernel::free.into_body(),
        0x78 => kernel::get_total_memory.into_body(),
        0x79 => kernel::get_free_memory.into_body(),
        0x7a => kernel::def_timer.into_body(),
        0x7b => kernel::set_timer.into_body(),
        0x7c => kernel::unset_timer.into_body(),
        0x7d => kernel::current_time.into_body(),
        0x7e => kernel::get_system_property.into_body(),
        0x7f => kernel::set_system_property.into_body(),
        0x80 => kernel::get_resource_id.into_body(),
        0x81 => kernel::get_resource.into_body(),
        0x97 => unk2.into_body(),
        0xc8 => graphics::get_image_property.into_body(),
        0xca => graphics::get_screen_framebuffer.into_body(),
        0xcb => graphics::destroy_offscreen_framebuffer.into_body(),
        0xcc => graphics::create_offscreen_framebuffer.into_body(),
        0xcd => graphics::init_context.into_body(),
        0xce => graphics::set_context.into_body(),
        0xd0 => graphics::put_pixel.into_body(),
        0xd3 => graphics::fill_rect.into_body(),
        0xd5 => graphics::draw_image.into_body(),
        0xda => graphics::draw_string.into_body(),
        0xde => graphics::flush.into_body(),
        0xdf => graphics::get_pixel_from_rgb.into_body(),
        0xe0 => graphics::get_rgb_from_pixel.into_body(),
        0xe1 => graphics::get_display_info.into_body(),
        0xe3 => graphics::get_font.into_body(),
        0xe4 => graphics::get_font_height.into_body(),
        0xe9 => graphics::create_image.into_body(),
        0xeb => unk0.into_body(),
        0x12c => unk3.into_body(),
        0x12d => unk4.into_body(),
        0x190 => database::open_database.into_body(),
        0x192 => database::write_record_single.into_body(),
        0x193 => database::close_database.into_body(),
        0x258 => net::connect.into_body(),
        0x259 => net::close.into_body(),
        0x25e => net::socket_close.into_body(),
        0x4b0 => media::clip_create.into_body(),
        0x4b1 => media::clip_free.into_body(),
        0x4b3 => media::clip_put_data.into_body(),
        0x4b8 => media::clip_get_volume.into_body(),
        0x4b9 => media::clip_set_volume.into_body(),
        0x4c0 => unk5.into_body(),
        0x4c1 => media::vibrator.into_body(),
        0x4c5 => media::clip_alloc_player.into_body(),
        0x4c6 => media::clip_free_player.into_body(),
        0x4d1 => media::set_mute_state.into_body(),
        0x4ba => media::play.into_body(),
        0x4bd => media::stop.into_body(),
        0x578 => misc::back_light.into_body(),
        _ => return Err(WieError::FatalError(format!("Unknown lgt wipi import: {function_index:#x}"))),
    };

    let mut context = LgtWIPICContext::new(core.clone(), system.clone(), jvm.clone());
    // lgt app calls get method only once per function, so it's okay to register function every time
    let address = context.register_function(method)?;

    Ok(address)
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct CletFunctions {
    start_clet: u32,
    pause_clet: u32,
    resume_clet: u32,
    destroy_clet: u32,
    paint_clet: u32,
    handle_clet_event: u32,
}

async fn clet_register(core: &mut ArmCore, jvm: &mut Jvm, function_table: u32, a1: u32) -> Result<()> {
    tracing::debug!("clet_register({:#x}, {:#x})", function_table, a1);

    let functions: CletFunctions = read_generic(core, function_table)?;
    tracing::info!("CletFunctions: {:x?}", functions);

    let context = CletWrapperContext { core: core.clone() };
    let clet_wrapper_class = ClassDefinitionImpl::from_class_proto(CletWrapper::as_proto(), Box::new(context.clone()) as Box<_>);
    let clet_wrapper_card_class = ClassDefinitionImpl::from_class_proto(CletWrapperCard::as_proto(), Box::new(context) as Box<_>);
    jvm.register_class(Box::new(clet_wrapper_class), None).await.unwrap();
    jvm.register_class(Box::new(clet_wrapper_card_class), None).await.unwrap();

    jvm.put_static_field("net/wie/CletWrapper", "startClet", "I", functions.start_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "pauseClet", "I", functions.pause_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "resumeClet", "I", functions.resume_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "destroyClet", "I", functions.destroy_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "paintClet", "I", functions.paint_clet as i32)
        .await
        .unwrap();
    jvm.put_static_field("net/wie/CletWrapper", "handleCletEvent", "I", functions.handle_clet_event as i32)
        .await
        .unwrap();

    let main_class_name = JavaLangString::from_rust_string(jvm, "net/wie/CletWrapper").await.unwrap();
    let mut args_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
    jvm.store_array(&mut args_array, 0, vec![main_class_name]).await.unwrap();

    let result: JvmResult<()> = jvm
        .invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args_array,))
        .await;

    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    Ok(())
}

async fn unk0(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk0({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    // graphics

    Ok(0)
}

async fn unk1(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk1({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    // kernel

    Ok(0)
}

async fn unk2(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::warn!("stub unk2");

    // OEMC_knlGetProgramInfo? get app id
    let result = context.alloc_raw(0x10)?;
    let app_id = context.system().app_id().to_string();
    write_null_terminated_string_bytes(context, result, app_id.as_bytes())?;

    Ok(result)
}

async fn unk3(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk3({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    Ok(0)
}

async fn unk4(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk4({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    Ok(0)
}

async fn unk5(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk5({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    // media

    Ok(0)
}
