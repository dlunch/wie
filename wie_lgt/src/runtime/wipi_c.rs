use alloc::{boxed::Box, string::ToString, vec};

mod context;

use jvm::{Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_rust::ClassDefinitionImpl;
use wipi_types::lgt::CletFunctions;
use wipi_types::wipic::WIPICIndirectPtr;

use wie_backend::System;
use wie_core_arm::{ArmCore, EmulatedFunction, EmulatedFunctionParam, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, read_generic, write_generic, write_null_terminated_string_bytes};
use wie_wipi_c::{
    MethodImpl, WIPICContext, WIPICMethodBody, WIPICResult,
    api::{database, graphics, kernel, media, misc, net},
};

use context::LgtWIPICContext;

use crate::runtime::java::classes::net::wie::{CletWrapper, CletWrapperCard, CletWrapperContext};
use crate::runtime::{SVC_CATEGORY_WIPIC, svc_ids::WIPICSvcId};

const TIME_VALUE_PTR: u32 = 0x7fff1004;

struct WIPICMethodResult {
    result: WIPICResult,
}

impl ResultWriter<WIPICMethodResult> for WIPICMethodResult {
    fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
        core.write_return_value(&self.result.results)?;
        core.set_next_pc(next_pc)?;

        Ok(())
    }
}

struct CMethodProxy {
    context: LgtWIPICContext,
    body: WIPICMethodBody,
}

async fn handle_wipic_svc(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), id: SvcId) -> Result<()> {
    let wipic_context = LgtWIPICContext::new(core.clone(), system.clone(), jvm.clone());
    let (_, lr) = core.read_pc_lr()?;
    let method = match WIPICSvcId::try_from(id)? {
        WIPICSvcId::CletRegister => {
            return EmulatedFunction::call(&clet_register, core, jvm).await?.write(core, lr);
        }
        WIPICSvcId::GetFramebufferPointer => graphics::get_framebuffer_pointer.into_body(),
        WIPICSvcId::GetFramebufferWidth => graphics::get_framebuffer_width.into_body(),
        WIPICSvcId::GetFramebufferHeight => graphics::get_framebuffer_height.into_body(),
        WIPICSvcId::GetFramebufferBpl => graphics::get_framebuffer_bpl.into_body(),
        WIPICSvcId::GetFramebufferBpp => graphics::get_framebuffer_bpp.into_body(),
        WIPICSvcId::Printk => kernel::printk.into_body(),
        WIPICSvcId::Sprintk => kernel::sprintk.into_body(),
        WIPICSvcId::Unk13 => unk13.into_body(),
        WIPICSvcId::Unk1 => unk1.into_body(),
        WIPICSvcId::Exit => kernel::exit.into_body(),
        WIPICSvcId::Alloc => kernel::alloc.into_body(),
        WIPICSvcId::Calloc => kernel::calloc.into_body(),
        WIPICSvcId::Free => kernel::free.into_body(),
        WIPICSvcId::GetTotalMemory => kernel::get_total_memory.into_body(),
        WIPICSvcId::GetFreeMemory => kernel::get_free_memory.into_body(),
        WIPICSvcId::DefTimer => kernel::def_timer.into_body(),
        WIPICSvcId::SetTimer => kernel::set_timer.into_body(),
        WIPICSvcId::UnsetTimer => kernel::unset_timer.into_body(),
        WIPICSvcId::CurrentTime => kernel::current_time.into_body(),
        WIPICSvcId::GetSystemProperty => kernel::get_system_property.into_body(),
        WIPICSvcId::SetSystemProperty => kernel::set_system_property.into_body(),
        WIPICSvcId::GetResourceId => kernel::get_resource_id.into_body(),
        WIPICSvcId::GetResource => kernel::get_resource.into_body(),
        WIPICSvcId::Unk2 => unk2.into_body(),
        WIPICSvcId::GetImageProperty => graphics::get_image_property.into_body(),
        WIPICSvcId::GetImageFramebuffer => graphics::get_image_framebuffer.into_body(),
        WIPICSvcId::GetScreenFramebuffer => graphics::get_screen_framebuffer.into_body(),
        WIPICSvcId::DestroyOffscreenFramebuffer => graphics::destroy_offscreen_framebuffer.into_body(),
        WIPICSvcId::CreateOffscreenFramebuffer => graphics::create_offscreen_framebuffer.into_body(),
        WIPICSvcId::InitContext => graphics::init_context.into_body(),
        WIPICSvcId::SetContext => graphics::set_context.into_body(),
        WIPICSvcId::PutPixel => graphics::put_pixel.into_body(),
        WIPICSvcId::DrawLine => graphics::draw_line.into_body(),
        WIPICSvcId::DrawRect => graphics::draw_rect.into_body(),
        WIPICSvcId::FillRect => graphics::fill_rect.into_body(),
        WIPICSvcId::CopyFrameBuffer => graphics::copy_frame_buffer.into_body(),
        WIPICSvcId::DrawImage => graphics::draw_image.into_body(),
        WIPICSvcId::CopyArea => graphics::copy_area.into_body(),
        WIPICSvcId::DrawString => graphics::draw_string.into_body(),
        WIPICSvcId::GetRgbPixels => graphics::get_rgb_pixels.into_body(),
        WIPICSvcId::SetRgbPixels => graphics::set_rgb_pixels.into_body(),
        WIPICSvcId::FlushLcd => graphics::flush_lcd.into_body(),
        WIPICSvcId::GetPixelFromRgb => graphics::get_pixel_from_rgb.into_body(),
        WIPICSvcId::GetRgbFromPixel => graphics::get_rgb_from_pixel.into_body(),
        WIPICSvcId::GetDisplayInfo => graphics::get_display_info.into_body(),
        WIPICSvcId::Repaint => graphics::repaint.into_body(),
        WIPICSvcId::GetFont => graphics::get_font.into_body(),
        WIPICSvcId::GetFontHeight => graphics::get_font_height.into_body(),
        WIPICSvcId::GetFontAscent => graphics::get_font_ascent.into_body(),
        WIPICSvcId::GetFontDescent => graphics::get_font_descent.into_body(),
        WIPICSvcId::GetStringWidth => graphics::get_string_width.into_body(),
        WIPICSvcId::CreateImage => graphics::create_image.into_body(),
        WIPICSvcId::Unk0 => unk0.into_body(),
        WIPICSvcId::Unk11 => unk11.into_body(),
        WIPICSvcId::Unk3 => unk3.into_body(),
        WIPICSvcId::Unk4 => unk4.into_body(),
        WIPICSvcId::Unk7 => unk7.into_body(),
        WIPICSvcId::Unk6 => unk6.into_body(),
        WIPICSvcId::TimeNow => time_now.into_body(),
        WIPICSvcId::TimeComponent => time_component.into_body(),
        WIPICSvcId::TimeConvert => time_convert.into_body(),
        WIPICSvcId::TimeToTm => time_to_tm.into_body(),
        WIPICSvcId::DateTimeToTm => time_to_tm.into_body(),
        WIPICSvcId::OpenDatabase => database::open_database.into_body(),
        WIPICSvcId::ReadRecordSingle => database::stream_read.into_body(),
        WIPICSvcId::WriteRecordSingle => database::stream_write.into_body(),
        WIPICSvcId::CloseDatabase => database::close_database.into_body(),
        WIPICSvcId::Unk12 => database::seek_record_single.into_body(),
        WIPICSvcId::Unk9 => database::list_record_info.into_body(),
        WIPICSvcId::DeleteRecord => database::delete_database.into_body(),
        WIPICSvcId::ListRecord => database::list_record.into_body(),
        WIPICSvcId::UpdateRecord => database::update_record.into_body(),
        WIPICSvcId::SelectRecord => database::select_record.into_body(),
        WIPICSvcId::Unk8 => database::exists_database.into_body(),
        WIPICSvcId::Connect => net::connect.into_body(),
        WIPICSvcId::Close => net::close.into_body(),
        WIPICSvcId::SocketClose => net::socket_close.into_body(),
        WIPICSvcId::ClipCreate => media::clip_create.into_body(),
        WIPICSvcId::ClipFree => media::clip_free.into_body(),
        WIPICSvcId::ClipPutData => media::clip_put_data.into_body(),
        WIPICSvcId::Unk15 => unk15.into_body(),
        WIPICSvcId::ClipGetVolume => media::clip_get_volume.into_body(),
        WIPICSvcId::ClipSetVolume => media::clip_set_volume.into_body(),
        WIPICSvcId::Play => media::play.into_body(),
        WIPICSvcId::Stop => media::stop.into_body(),
        WIPICSvcId::Unk5 => unk5.into_body(),
        WIPICSvcId::Vibrator => media::vibrator.into_body(),
        WIPICSvcId::Unk14 => unk14.into_body(),
        WIPICSvcId::ClipAllocPlayer => media::clip_alloc_player.into_body(),
        WIPICSvcId::ClipFreePlayer => media::clip_free_player.into_body(),
        WIPICSvcId::Unk10 => unk10.into_body(),
        WIPICSvcId::SetMuteState => media::set_mute_state.into_body(),
        WIPICSvcId::GetMuteState => media::get_mute_state.into_body(),
        WIPICSvcId::BackLight => misc::back_light.into_body(),
    };

    EmulatedFunction::call(
        &CMethodProxy {
            context: wipic_context,
            body: method,
        },
        core,
        &mut (),
    )
    .await?
    .write(core, lr)
}

#[async_trait::async_trait]
impl EmulatedFunction<(), WIPICMethodResult, ()> for CMethodProxy {
    async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<WIPICMethodResult> {
        let a0 = u32::get(core, 0);
        let a1 = u32::get(core, 1);
        let a2 = u32::get(core, 2);
        let a3 = u32::get(core, 3);
        let a4 = u32::get(core, 4);
        let a5 = u32::get(core, 5);
        let a6 = u32::get(core, 6);
        let a7 = u32::get(core, 7);
        let a8 = u32::get(core, 8);

        let result = self
            .body
            .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
            .await?;

        Ok(WIPICMethodResult { result })
    }
}

pub fn register_wipic_svc_handler(core: &mut ArmCore, system: &System, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_WIPIC, handle_wipic_svc, &(system.clone(), jvm.clone()))
}

async fn clet_register(core: &mut ArmCore, jvm: &mut Jvm, function_table: u32, a1: u32) -> Result<()> {
    tracing::debug!("clet_register({function_table:#x}, {a1:#x})");

    let functions: CletFunctions = read_generic(core, function_table)?;

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

    invoke_lcdui_main(jvm, "net/wie/CletWrapper").await
}

/// Shared boot path: invoke `org.kwis.msp.lcdui.Main.main` with a single-element
/// `String[]` whose `[0]` is the Jlet/MIDlet main class name. `Main.main`
/// instantiates that class and drives its lifecycle.
///
/// Used by both the WIPI-C clet path (`clet_register`, main class
/// `net/wie/CletWrapper`) and the Java-interface boot path (`java_unk11`).
pub(crate) async fn invoke_lcdui_main(jvm: &mut Jvm, main_class_name: &str) -> Result<()> {
    let main_class_name = JavaLangString::from_rust_string(jvm, main_class_name).await.unwrap();
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
    tracing::warn!("stub unk0({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // graphics

    Ok(0)
}

async fn unk1(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk1({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // kernel

    Ok(0)
}

async fn unk2(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::warn!("stub unk2");

    // OEMC_knlGetProgramInfo? get app id
    let app_id = context.system().aid().to_string();
    let result = context.alloc_raw((app_id.len() + 1) as u32)?;
    write_null_terminated_string_bytes(context, result, app_id.as_bytes())?;

    Ok(result)
}

async fn unk3(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk3({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk4(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk4({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk5(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk5({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}

async fn unk6(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk6({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk7(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk7({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn time_now(context: &mut dyn WIPICContext, component_class: u32) -> Result<u32> {
    let epoch_seconds = context.system().platform().now().raw() / 1000;
    tracing::debug!("LGT_timeNow({component_class:#x}) -> {epoch_seconds}");

    write_time_value(context, epoch_seconds as u32)
}

async fn time_component(_context: &mut dyn WIPICContext, name: u32) -> Result<u32> {
    tracing::debug!("LGT_timeComponent({name:#x})");

    Ok(name)
}

async fn time_convert(context: &mut dyn WIPICContext, date_time: u32, component: u32) -> Result<u32> {
    tracing::debug!("LGT_timeConvert({date_time:#x}, {component:#x})");

    let timestamp = read_time_value(context, date_time)?;
    write_time_value(context, timestamp)
}

async fn time_to_tm(context: &mut dyn WIPICContext, time_value: u32, out_ptr: u32) -> Result<i32> {
    tracing::debug!("LGT_timeToTm({time_value:#x}, {out_ptr:#x})");

    let timestamp = read_time_value(context, time_value)?;
    let (year, month, day, hour, minute, second) = unix_seconds_to_utc(timestamp as i64);
    write_generic(context, out_ptr, second)?;
    write_generic(context, out_ptr + 4, minute)?;
    write_generic(context, out_ptr + 8, hour)?;
    write_generic(context, out_ptr + 12, day)?;
    write_generic(context, out_ptr + 16, month - 1)?;
    write_generic(context, out_ptr + 20, year - 1900)?;

    Ok(0)
}

fn write_time_value(context: &mut dyn WIPICContext, timestamp: u32) -> Result<u32> {
    let time_value_ptr: u32 = read_generic(context, TIME_VALUE_PTR)?;
    let memory = if time_value_ptr != 0 {
        WIPICIndirectPtr(time_value_ptr)
    } else {
        let memory = context.alloc(4)?;
        write_generic(context, TIME_VALUE_PTR, memory.0)?;
        memory
    };
    write_generic(context, context.data_ptr(memory)?, timestamp)?;
    Ok(memory.0)
}

fn read_time_value(context: &mut dyn WIPICContext, handle: u32) -> Result<u32> {
    read_generic(context, context.data_ptr(WIPICIndirectPtr(handle))?)
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

async fn unk10(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk10({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk11(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk11({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

async fn unk13(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk13({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // kernel

    Ok(0)
}

async fn unk14(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk14({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}

async fn unk15(_context: &mut dyn WIPICContext, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::warn!("stub unk15({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    // media

    Ok(0)
}
