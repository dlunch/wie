use alloc::{boxed::Box, format, vec};

mod context;

use bytemuck::{Pod, Zeroable};

use jvm::{runtime::JavaLangString, Jvm, Result as JvmResult};
use jvm_rust::ClassDefinitionImpl;

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_jvm_support::JvmSupport;
use wie_util::{read_generic, Result, WieError};
use wie_wipi_c::{
    api::{database, graphics, kernel, misc},
    MethodImpl, WIPICContext,
};

use context::LgtWIPICContext;

use crate::runtime::classes::net::wie::{CletWrapper, CletWrapperCard, CletWrapperContext};

pub fn get_wipi_c_method(core: &mut ArmCore, system: &mut System, jvm: &Jvm, function_index: u32) -> Result<u32> {
    let method = match function_index {
        0x03 => return core.register_function(clet_register, jvm),
        0x32 => unk1.into_body(),
        0x33 => unk2.into_body(),
        0x34 => unk3.into_body(),
        0x64 => kernel::printk.into_body(),
        0x65 => kernel::sprintk.into_body(),
        0x75 => kernel::alloc.into_body(),
        0x76 => kernel::calloc.into_body(),
        0x77 => kernel::free.into_body(),
        0x79 => kernel::get_free_memory.into_body(),
        0x7a => kernel::def_timer.into_body(),
        0x7b => kernel::set_timer.into_body(),
        0x7d => kernel::current_time.into_body(),
        0x7e => kernel::get_system_property.into_body(),
        0x7f => kernel::set_system_property.into_body(),
        0x80 => kernel::get_resource_id.into_body(),
        0x81 => kernel::get_resource.into_body(),
        0xca => graphics::get_screen_framebuffer.into_body(),
        0xcd => graphics::init_context.into_body(),
        0xce => graphics::set_context.into_body(),
        0xd0 => graphics::put_pixel.into_body(),
        0xdf => graphics::get_pixel_from_rgb.into_body(),
        0xe1 => graphics::get_display_info.into_body(),
        0xe3 => graphics::get_font.into_body(),
        0xe4 => graphics::get_font_height.into_body(),
        0xe9 => graphics::create_image.into_body(),
        0x190 => database::open_database.into_body(),
        0x4b9 => unk0.into_body(),
        0x578 => misc::back_light.into_body(),
        _ => return Err(WieError::FatalError(format!("Unknown lgt wipi import: {:#x}", function_index))),
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
    handle_input: u32,
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
    jvm.put_static_field("net/wie/CletWrapper", "handleInput", "I", functions.handle_input as i32)
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

async fn unk0(_context: &mut dyn WIPICContext, a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk0({:#x}, {:#x})", a0, a1);

    // maps to OEMC_mdaClipGetInfo on ktf but seems wrong

    Ok(())
}

async fn unk1(_context: &mut dyn WIPICContext, a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk1({:#x}, {:#x})", a0, a1);

    Ok(())
}

async fn unk2(_context: &mut dyn WIPICContext, a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk2({:#x}, {:#x})", a0, a1);

    Ok(())
}

async fn unk3(_context: &mut dyn WIPICContext, a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk3({:#x}, {:#x})", a0, a1);

    Ok(())
}
