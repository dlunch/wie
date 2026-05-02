use alloc::{format, vec, vec::Vec};

use wipi_types::{
    ktf::wipic::{WIPICDatabaseInterface, WIPICGraphicsInterface, WIPICKnlInterface},
    wipic::WIPICWord,
};

use wie_core_arm::ArmCore;
use wie_util::{Result, WieError};
use wie_wipi_c::{
    MethodImpl, WIPICContext, WIPICMethodBody,
    api::{database, graphics, kernel, media, misc, net, uic, util},
};

use crate::runtime::{
    SVC_CATEGORY_WIPIC,
    svc_ids::{WIPICDatabaseMethodId, WIPICGraphicsMethodId, WIPICKernelMethodId, WIPICTableId},
};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WieError::Unimplemented(format!("{id}: {name}"))) };

    body.into_body()
}

pub fn get_kernel_interface(core: &mut ArmCore) -> Result<WIPICKnlInterface> {
    let table_id = WIPICTableId::Kernel;

    Ok(WIPICKnlInterface {
        printk: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Printk))?,
        sprintk: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Sprintk))?,
        get_exec_names: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetExecNames))?,
        execute: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Execute))?,
        mexecute: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Mexecute))?,
        load: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Load))?,
        mload: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Mload))?,
        exit: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Exit))?,
        program_stop: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::ProgramStop))?,
        get_cur_program_id: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetCurProgramId))?,
        get_parent_program_id: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetParentProgramId))?,
        get_app_manager_id: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetAppManagerId))?,
        get_program_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetProgramInfo))?,
        get_access_level: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetAccessLevel))?,
        get_program_name: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetProgramName))?,
        create_shared_buf: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CreateSharedBuf))?,
        destroy_shared_buf: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::DestroySharedBuf))?,
        get_shared_buf: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetSharedBuf))?,
        get_shared_buf_size: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetSharedBufSize))?,
        resize_shared_buf: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::ResizeSharedBuf))?,
        alloc: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Alloc))?,
        calloc: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Calloc))?,
        free: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Free))?,
        get_total_memory: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetTotalMemory))?,
        get_free_memory: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetFreeMemory))?,
        def_timer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::DefTimer))?,
        set_timer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::SetTimer))?,
        unset_timer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::UnsetTimer))?,
        current_time: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CurrentTime))?,
        get_system_property: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetSystemProperty))?,
        set_system_property: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::SetSystemProperty))?,
        get_resource_id: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetResourceId))?,
        get_resource: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetResource))?,
        reserved1: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved1))?,
        reserved2: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved2))?,
        reserved3: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved3))?,
        reserved4: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved4))?,
        reserved5: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved5))?,
        reserved6: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved6))?,
        reserved7: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved7))?,
        reserved8: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved8))?,
        reserved9: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved9))?,
        reserved10: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved10))?,
        reserved11: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved11))?,
        send_message: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::SendMessage))?,
        set_timer_ex: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::SetTimerEx))?,
        get_system_state: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetSystemState))?,
        create_system_progress_bar: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CreateSystemProgressBar))?,
        set_system_progress_bar: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::SetSystemProgressBar))?,
        destroy_system_progress_bar: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::DestroySystemProgressBar))?,
        execute_ex: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::ExecuteEx))?,
        get_proc_address: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetProcAddress))?,
        unload: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Unload))?,
        create_sys_message_box: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CreateSysMessageBox))?,
        destroy_sys_message_box: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::DestroySysMessageBox))?,
        get_program_id_list: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetProgramIdList))?,
        get_program_info2: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetProgramInfo2))?,
        reserved12: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved12))?,
        reserved13: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::Reserved13))?,
        create_app_private_area: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CreateAppPrivateArea))?,
        get_app_private_area: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetAppPrivateArea))?,
        create_lib_private_area: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::CreateLibPrivateArea))?,
        get_lib_private_area: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetLibPrivateArea))?,
        get_platform_version: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetPlatformVersion))?,
        get_token: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICKernelMethodId::GetToken))?,
    })
}

pub fn get_graphics_interface(core: &mut ArmCore) -> Result<WIPICGraphicsInterface> {
    let table_id = WIPICTableId::Graphics;

    Ok(WIPICGraphicsInterface {
        get_image_property: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetImageProperty))?,
        get_image_framebuffer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetImageFramebuffer))?,
        get_screen_framebuffer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetScreenFramebuffer))?,
        destroy_offscreen_framebuffer: core.make_svc_stub(
            SVC_CATEGORY_WIPIC,
            table_id.function_id(WIPICGraphicsMethodId::DestroyOffscreenFramebuffer),
        )?,
        create_offscreen_framebuffer: core.make_svc_stub(
            SVC_CATEGORY_WIPIC,
            table_id.function_id(WIPICGraphicsMethodId::CreateOffscreenFramebuffer),
        )?,
        init_context: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::InitContext))?,
        set_context: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetContext))?,
        get_context: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetContext))?,
        put_pixel: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::PutPixel))?,
        draw_line: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawLine))?,
        draw_rect: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawRect))?,
        fill_rect: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::FillRect))?,
        copy_frame_buffer: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::CopyFrameBuffer))?,
        draw_image: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawImage))?,
        copy_area: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::CopyArea))?,
        draw_arc: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawArc))?,
        fill_arc: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::FillArc))?,
        draw_string: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawString))?,
        draw_unicode_string: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawUnicodeString))?,
        get_rgb_pixels: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetRgbPixels))?,
        set_rgb_pixels: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetRgbPixels))?,
        flush_lcd: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::FlushLcd))?,
        get_pixel_from_rgb: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetPixelFromRgb))?,
        get_rgb_from_pixel: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetRgbFromPixel))?,
        get_display_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetDisplayInfo))?,
        repaint: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::Repaint))?,
        get_font: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFont))?,
        get_font_height: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontHeight))?,
        get_font_ascent: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontAscent))?,
        get_font_descent: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontDescent))?,
        get_string_width: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetStringWidth))?,
        get_unicode_string_width: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetUnicodeStringWidth))?,
        create_image: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::CreateImage))?,
        destroy_image: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DestroyImage))?,
        decode_next_image: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DecodeNextImage))?,
        encode_image: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::EncodeImage))?,
        post_event: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::PostEvent))?,
        handle_input: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::HandleInput))?,
        set_current_mode: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetCurrentMode))?,
        get_current_mode: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetCurrentMode))?,
        get_support_mode_count: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetSupportModeCount))?,
        get_supported_modes: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetSupportedModes))?,
        fill_polygon: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::FillPolygon))?,
        draw_polygon: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::DrawPolygon))?,
        show_annunciator: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::ShowAnnunciator))?,
        get_annunciator_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetAnnunciatorInfo))?,
        set_annunciator_icon: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetAnnunciatorIcon))?,
        get_idle_help_line_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetIdleHelpLineInfo))?,
        show_help_line: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::ShowHelpLine))?,
        get_char_glyph: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetCharGlyph))?,
        create_image_ex: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::CreateImageEx))?,
        hide_help_line: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::HideHelpLine))?,
        set_clone_screen_framebuffer: core
            .make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetCloneScreenFramebuffer))?,
        get_font_ex: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontEx))?,
        get_font_lists: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontLists))?,
        get_font_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontInfo))?,
        set_font_help_line: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::SetFontHelpLine))?,
        get_font_help_line: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetFontHelpLine))?,
        encode_image_ex: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::EncodeImageEx))?,
        get_image_info: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICGraphicsMethodId::GetImageInfo))?,
    })
}

pub fn get_util_method_table() -> Vec<WIPICMethodBody> {
    vec![
        gen_stub(0, "MC_utilHtonl"),
        util::htons.into_body(),
        gen_stub(2, "MC_utilNtohl"),
        gen_stub(3, "MC_utilNtohs"),
        gen_stub(4, "MC_utilInetAddrInt"),
        gen_stub(5, "MC_utilInetAddrStr"),
        gen_stub(6, "OEMC_utilHashbySHA1"),
    ]
}

pub fn get_misc_method_table() -> Vec<WIPICMethodBody> {
    vec![
        misc::back_light.into_body(),
        gen_stub(1, "MC_miscSetLed"),
        gen_stub(2, "MC_miscGetLed"),
        gen_stub(3, "MC_miscGetLedCount"),
        gen_stub(4, "OEMC_miscGetCompassData"),
    ]
}

pub fn get_database_interface(core: &mut ArmCore) -> Result<WIPICDatabaseInterface> {
    let table_id = WIPICTableId::Database;

    Ok(WIPICDatabaseInterface {
        open_database: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::OpenDatabase))?,
        read_record_single: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::StreamRead))?,
        write_record_single: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::StreamWrite))?,
        close_database: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::CloseDatabase))?,
        select_record: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::SelectRecord))?,
        update_record: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::UpdateRecord))?,
        delete_record: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::DeleteRecord))?,
        list_record: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::ListRecord))?,
        sort_records: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::SortRecords))?,
        get_access_mode: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::GetAccessMode))?,
        get_number_of_records: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::GetNumberOfRecords))?,
        get_record_size: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::GetRecordSize))?,
        list_databases: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::ListDatabases))?,
        unk13: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::Unk13))?,
        unk14: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::Unk14))?,
        unk15: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::Unk15))?,
        unk16: core.make_svc_stub(SVC_CATEGORY_WIPIC, table_id.function_id(WIPICDatabaseMethodId::Unk16))?,
    })
}

pub fn get_uic_method_table() -> Vec<WIPICMethodBody> {
    vec![
        uic::create_application_context.into_body(),
        uic::get_class.into_body(),
        uic::create.into_body(),
        uic::destroy.into_body(),
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
        uic::get_menu_item.into_body(),
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

pub fn get_media_method_table() -> Vec<WIPICMethodBody> {
    vec![
        media::clip_create.into_body(),
        gen_stub(1, "MC_mdaUnk1"),
        gen_stub(2, "MC_mdaUnk2"),
        media::clip_free.into_body(),
        media::clip_put_data.into_body(),
        gen_stub(5, "MC_mdaUnk5"),
        gen_stub(6, "MC_mdaUnk6"),
        media::unk7.into_body(),
        media::play.into_body(),
        media::pause.into_body(),
        media::resume.into_body(),
        media::stop.into_body(),
        gen_stub(12, "MC_mdaUnk12"),
        gen_stub(13, "MC_mdaUnk13"),
        media::get_volume.into_body(),
        gen_stub(15, "MC_mdaUnk15"),
        media::vibrator.into_body(),
        media::unk17.into_body(),
        media::unk18.into_body(),
        gen_stub(19, "MC_mdaUnk19"),
        gen_stub(20, "MC_mdaUnk20"),
        gen_stub(21, "MC_mdaUnk21"),
        gen_stub(22, "MC_mdaUnk22"),
        gen_stub(23, "MC_mdaUnk23"),
        gen_stub(24, "MC_mdaUnk24"),
        media::clip_get_volume.into_body(),
        media::clip_set_volume.into_body(),
    ]
}

pub fn get_net_method_table() -> Vec<WIPICMethodBody> {
    vec![
        net::connect.into_body(),
        net::close.into_body(),
        gen_stub(2, "MC_netSocket"),
        gen_stub(3, "MC_netSocketConnect"),
        gen_stub(4, "MC_netSocketWrite"),
        gen_stub(5, "MC_netSocketRead"),
        net::socket_close.into_body(),
        gen_stub(7, "MC_netSocketBind"),
        gen_stub(8, "MC_netGetMaxPacketLength"),
        gen_stub(9, "MC_netSocketSendTo"),
        gen_stub(10, "MC_netSocketRcvFrom"),
        gen_stub(11, "MC_netGetHostAddr"),
        gen_stub(12, "MC_netSocketAccept"),
        gen_stub(13, "MC_netSetReadCB"),
        gen_stub(14, "MC_netSetWriteCB"),
        gen_stub(15, "MC_netHttpOpen"),
        gen_stub(16, "MC_netHttpConnect"),
        gen_stub(17, "MC_netHttpSetRequestMethod"),
        gen_stub(18, "MC_netHttpGetRequestMethod"),
        gen_stub(19, "MC_netHttpSetRequestProperty"),
        gen_stub(20, "MC_netHttpGetRequestProperty"),
        gen_stub(21, "MC_netHttpSetProxy"),
        gen_stub(22, "MC_netHttpGetProxy"),
        gen_stub(23, "MC_netHttpGetResponseCode"),
        gen_stub(24, "MC_netHttpGetResponseMessage"),
        gen_stub(25, "MC_netHttpGetHeaderField"),
        gen_stub(26, "MC_netHttpGetLength"),
        gen_stub(27, "MC_netHttpGetType"),
        gen_stub(28, "MC_netHttpGetEncoding"),
        gen_stub(29, "MC_netHttpClose"),
    ]
}

fn gen_unk_stub(id: u32, index: u32) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move {
        tracing::warn!("stub unk{id}-{index}");
        Ok::<u32, _>(0)
    };

    body.into_body()
}

pub fn get_unk3_method_table() -> Vec<WIPICMethodBody> {
    vec![
        gen_unk_stub(3, 0),
        gen_unk_stub(3, 1),
        gen_unk_stub(3, 2),
        gen_unk_stub(3, 3),
        gen_unk_stub(3, 4),
    ]
}

pub fn get_unk12_method_table() -> Vec<WIPICMethodBody> {
    vec![gen_unk_stub(12, 0), gen_unk_stub(12, 1), gen_unk_stub(12, 2)]
}

pub fn get_stub_method_table(interface: WIPICWord) -> Vec<WIPICMethodBody> {
    (0..64).map(|_| gen_stub(interface, "stub")).collect::<Vec<_>>()
}

pub fn get_method_body(table_id: WIPICTableId, function_id: u16) -> Option<WIPICMethodBody> {
    match table_id {
        WIPICTableId::Kernel => match WIPICKernelMethodId::try_from(function_id).ok()? {
            WIPICKernelMethodId::Printk => Some(kernel::printk.into_body()),
            WIPICKernelMethodId::Sprintk => Some(kernel::sprintk.into_body()),
            WIPICKernelMethodId::GetExecNames => Some(gen_stub(2, "MC_knlGetExecNames")),
            WIPICKernelMethodId::Execute => Some(gen_stub(3, "MC_knlExecute")),
            WIPICKernelMethodId::Mexecute => Some(gen_stub(4, "MC_knlMExecute")),
            WIPICKernelMethodId::Load => Some(gen_stub(5, "MC_knlLoad")),
            WIPICKernelMethodId::Mload => Some(gen_stub(6, "MC_knlMLoad")),
            WIPICKernelMethodId::Exit => Some(kernel::exit.into_body()),
            WIPICKernelMethodId::ProgramStop => Some(gen_stub(8, "MC_knlProgramStop")),
            WIPICKernelMethodId::GetCurProgramId => Some(kernel::get_cur_program_id.into_body()),
            WIPICKernelMethodId::GetParentProgramId => Some(gen_stub(10, "MC_knlGetParentProgramID")),
            WIPICKernelMethodId::GetAppManagerId => Some(gen_stub(11, "MC_knlGetAppManagerID")),
            WIPICKernelMethodId::GetProgramInfo => Some(gen_stub(12, "MC_knlGetProgramInfo")),
            WIPICKernelMethodId::GetAccessLevel => Some(gen_stub(13, "MC_knlGetAccessLevel")),
            WIPICKernelMethodId::GetProgramName => Some(kernel::get_program_name.into_body()),
            WIPICKernelMethodId::CreateSharedBuf => Some(gen_stub(15, "MC_knlCreateSharedBuf")),
            WIPICKernelMethodId::DestroySharedBuf => Some(gen_stub(16, "MC_knlDestroySharedBuf")),
            WIPICKernelMethodId::GetSharedBuf => Some(gen_stub(17, "MC_knlGetSharedBuf")),
            WIPICKernelMethodId::GetSharedBufSize => Some(gen_stub(18, "MC_knlGetSharedBufSize")),
            WIPICKernelMethodId::ResizeSharedBuf => Some(gen_stub(19, "MC_knlResizeSharedBuf")),
            WIPICKernelMethodId::Alloc => Some(kernel::alloc.into_body()),
            WIPICKernelMethodId::Calloc => Some(kernel::calloc.into_body()),
            WIPICKernelMethodId::Free => Some(kernel::free.into_body()),
            WIPICKernelMethodId::GetTotalMemory => Some(kernel::get_total_memory.into_body()),
            WIPICKernelMethodId::GetFreeMemory => Some(kernel::get_free_memory.into_body()),
            WIPICKernelMethodId::DefTimer => Some(kernel::def_timer.into_body()),
            WIPICKernelMethodId::SetTimer => Some(kernel::set_timer.into_body()),
            WIPICKernelMethodId::UnsetTimer => Some(kernel::unset_timer.into_body()),
            WIPICKernelMethodId::CurrentTime => Some(kernel::current_time.into_body()),
            WIPICKernelMethodId::GetSystemProperty => Some(kernel::get_system_property.into_body()),
            WIPICKernelMethodId::SetSystemProperty => Some(kernel::set_system_property.into_body()),
            WIPICKernelMethodId::GetResourceId => Some(kernel::get_resource_id.into_body()),
            WIPICKernelMethodId::GetResource => Some(kernel::get_resource.into_body()),
            WIPICKernelMethodId::Reserved1 => None,
            WIPICKernelMethodId::Reserved2 => Some(gen_stub(34, "MC_knlReserved2")),
            WIPICKernelMethodId::Reserved3 => Some(gen_stub(35, "MC_knlReserved3")),
            WIPICKernelMethodId::Reserved4 => Some(gen_stub(36, "MC_knlReserved4")),
            WIPICKernelMethodId::Reserved5 => Some(gen_stub(37, "MC_knlReserved5")),
            WIPICKernelMethodId::Reserved6 => Some(gen_stub(38, "MC_knlReserved6")),
            WIPICKernelMethodId::Reserved7 => Some(gen_stub(39, "MC_knlReserved7")),
            WIPICKernelMethodId::Reserved8 => Some(gen_stub(40, "MC_knlReserved8")),
            WIPICKernelMethodId::Reserved9 => Some(gen_stub(41, "MC_knlReserved9")),
            WIPICKernelMethodId::Reserved10 => Some(gen_stub(42, "MC_knlReserved10")),
            WIPICKernelMethodId::Reserved11 => Some(gen_stub(43, "MC_knlReserved11")),
            WIPICKernelMethodId::SendMessage => Some(gen_stub(44, "OEMC_knlSendMessage")),
            WIPICKernelMethodId::SetTimerEx => Some(gen_stub(45, "OEMC_knlSetTimerEx")),
            WIPICKernelMethodId::GetSystemState => Some(gen_stub(46, "OEMC_knlGetSystemState")),
            WIPICKernelMethodId::CreateSystemProgressBar => Some(gen_stub(47, "OEMC_knlCreateSystemProgressBar")),
            WIPICKernelMethodId::SetSystemProgressBar => Some(gen_stub(48, "OEMC_knlSetSystemProgressBar")),
            WIPICKernelMethodId::DestroySystemProgressBar => Some(gen_stub(49, "OEMC_knlDestroySystemProgressBar")),
            WIPICKernelMethodId::ExecuteEx => Some(gen_stub(50, "OEMC_knlExecuteEx")),
            WIPICKernelMethodId::GetProcAddress => Some(gen_stub(51, "OEMC_knlGetProcAddress")),
            WIPICKernelMethodId::Unload => Some(gen_stub(52, "OEMC_knlUnload")),
            WIPICKernelMethodId::CreateSysMessageBox => Some(gen_stub(53, "OEMC_knlCreateSysMessageBox")),
            WIPICKernelMethodId::DestroySysMessageBox => Some(gen_stub(54, "OEMC_knlDestroySysMessageBox")),
            WIPICKernelMethodId::GetProgramIdList => Some(gen_stub(55, "OEMC_knlGetProgramIDList")),
            WIPICKernelMethodId::GetProgramInfo2 => Some(gen_stub(56, "OEMC_knlGetProgramInfo")),
            WIPICKernelMethodId::Reserved12 => Some(gen_stub(57, "MC_knlReserved12")),
            WIPICKernelMethodId::Reserved13 => Some(gen_stub(58, "MC_knlReserved13")),
            WIPICKernelMethodId::CreateAppPrivateArea => Some(gen_stub(59, "OEMC_knlCreateAppPrivateArea")),
            WIPICKernelMethodId::GetAppPrivateArea => Some(gen_stub(60, "OEMC_knlGetAppPrivateArea")),
            WIPICKernelMethodId::CreateLibPrivateArea => Some(gen_stub(61, "OEMC_knlCreateLibPrivateArea")),
            WIPICKernelMethodId::GetLibPrivateArea => Some(gen_stub(62, "OEMC_knlGetLibPrivateArea")),
            WIPICKernelMethodId::GetPlatformVersion => Some(gen_stub(63, "OEMC_knlGetPlatformVersion")),
            WIPICKernelMethodId::GetToken => Some(gen_stub(64, "OEMC_knlGetToken")),
        },
        WIPICTableId::Util => get_util_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Misc => get_misc_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Graphics => match WIPICGraphicsMethodId::try_from(function_id).ok()? {
            WIPICGraphicsMethodId::GetImageProperty => Some(graphics::get_image_property.into_body()),
            WIPICGraphicsMethodId::GetImageFramebuffer => Some(graphics::get_image_framebuffer.into_body()),
            WIPICGraphicsMethodId::GetScreenFramebuffer => Some(graphics::get_screen_framebuffer.into_body()),
            WIPICGraphicsMethodId::DestroyOffscreenFramebuffer => Some(graphics::destroy_offscreen_framebuffer.into_body()),
            WIPICGraphicsMethodId::CreateOffscreenFramebuffer => Some(graphics::create_offscreen_framebuffer.into_body()),
            WIPICGraphicsMethodId::InitContext => Some(graphics::init_context.into_body()),
            WIPICGraphicsMethodId::SetContext => Some(graphics::set_context.into_body()),
            WIPICGraphicsMethodId::GetContext => Some(gen_stub(7, "MC_grpGetContext")),
            WIPICGraphicsMethodId::PutPixel => Some(graphics::put_pixel.into_body()),
            WIPICGraphicsMethodId::DrawLine => Some(graphics::draw_line.into_body()),
            WIPICGraphicsMethodId::DrawRect => Some(graphics::draw_rect.into_body()),
            WIPICGraphicsMethodId::FillRect => Some(graphics::fill_rect.into_body()),
            WIPICGraphicsMethodId::CopyFrameBuffer => Some(graphics::copy_frame_buffer.into_body()),
            WIPICGraphicsMethodId::DrawImage => Some(graphics::draw_image.into_body()),
            WIPICGraphicsMethodId::CopyArea => Some(graphics::copy_area.into_body()),
            WIPICGraphicsMethodId::DrawArc => Some(gen_stub(15, "MC_grpDrawArc")),
            WIPICGraphicsMethodId::FillArc => Some(gen_stub(16, "MC_grpFillArc")),
            WIPICGraphicsMethodId::DrawString => Some(graphics::draw_string.into_body()),
            WIPICGraphicsMethodId::DrawUnicodeString => Some(gen_stub(18, "MC_grpDrawUnicodeString")),
            WIPICGraphicsMethodId::GetRgbPixels => Some(graphics::get_rgb_pixels.into_body()),
            WIPICGraphicsMethodId::SetRgbPixels => Some(graphics::set_rgb_pixels.into_body()),
            WIPICGraphicsMethodId::FlushLcd => Some(graphics::flush_lcd.into_body()),
            WIPICGraphicsMethodId::GetPixelFromRgb => Some(graphics::get_pixel_from_rgb.into_body()),
            WIPICGraphicsMethodId::GetRgbFromPixel => Some(graphics::get_rgb_from_pixel.into_body()),
            WIPICGraphicsMethodId::GetDisplayInfo => Some(graphics::get_display_info.into_body()),
            WIPICGraphicsMethodId::Repaint => Some(graphics::repaint.into_body()),
            WIPICGraphicsMethodId::GetFont => Some(graphics::get_font.into_body()),
            WIPICGraphicsMethodId::GetFontHeight => Some(graphics::get_font_height.into_body()),
            WIPICGraphicsMethodId::GetFontAscent => Some(graphics::get_font_ascent.into_body()),
            WIPICGraphicsMethodId::GetFontDescent => Some(graphics::get_font_descent.into_body()),
            WIPICGraphicsMethodId::GetStringWidth => Some(graphics::get_string_width.into_body()),
            WIPICGraphicsMethodId::GetUnicodeStringWidth => Some(gen_stub(31, "MC_grpGetUnicodeStringWidth")),
            WIPICGraphicsMethodId::CreateImage => Some(graphics::create_image.into_body()),
            WIPICGraphicsMethodId::DestroyImage => Some(graphics::destroy_image.into_body()),
            WIPICGraphicsMethodId::DecodeNextImage => Some(gen_stub(34, "MC_grpDecodeNextImage")),
            WIPICGraphicsMethodId::EncodeImage => Some(gen_stub(35, "MC_grpEncodeImage")),
            WIPICGraphicsMethodId::PostEvent => Some(graphics::post_event.into_body()),
            WIPICGraphicsMethodId::HandleInput => Some(gen_stub(37, "MC_imHandleInput")),
            WIPICGraphicsMethodId::SetCurrentMode => Some(gen_stub(38, "MC_imSetCurrentMode")),
            WIPICGraphicsMethodId::GetCurrentMode => Some(gen_stub(39, "MC_imGetCurrentMode")),
            WIPICGraphicsMethodId::GetSupportModeCount => Some(gen_stub(40, "MC_imGetSupportModeCount")),
            WIPICGraphicsMethodId::GetSupportedModes => Some(gen_stub(41, "MC_imGetSupportedModes")),
            WIPICGraphicsMethodId::FillPolygon => Some(gen_stub(42, "MC_grpFillPolygon")),
            WIPICGraphicsMethodId::DrawPolygon => Some(gen_stub(43, "MC_grpDrawPolygon")),
            WIPICGraphicsMethodId::ShowAnnunciator => Some(gen_stub(44, "OEMC_grpShowAnnunciator")),
            WIPICGraphicsMethodId::GetAnnunciatorInfo => Some(gen_stub(45, "OEMC_grpGetAnnunciatorInfo")),
            WIPICGraphicsMethodId::SetAnnunciatorIcon => Some(gen_stub(46, "OEMC_grp  SetAnnunciatorIcon")),
            WIPICGraphicsMethodId::GetIdleHelpLineInfo => Some(gen_stub(47, "OEMC_grpGetIdleHelpLineInfo")),
            WIPICGraphicsMethodId::ShowHelpLine => Some(gen_stub(48, "OEMC_grpShowHelpLine")),
            WIPICGraphicsMethodId::GetCharGlyph => Some(gen_stub(49, "OEMC_grpGetCharGlyph")),
            WIPICGraphicsMethodId::CreateImageEx => Some(gen_stub(50, "OEMC_grpCreateImageEx")),
            WIPICGraphicsMethodId::HideHelpLine => Some(gen_stub(51, "OEMC_grpHideHelpLine")),
            WIPICGraphicsMethodId::SetCloneScreenFramebuffer => Some(gen_stub(52, "OEMC_grpSetCloneScreenFrameBuffer")),
            WIPICGraphicsMethodId::GetFontEx => Some(gen_stub(53, "OEMC_grpGetFontEx")),
            WIPICGraphicsMethodId::GetFontLists => Some(gen_stub(54, "OEMC_grpGetFontLists")),
            WIPICGraphicsMethodId::GetFontInfo => Some(gen_stub(55, "OEMC_grpGetFontInfo")),
            WIPICGraphicsMethodId::SetFontHelpLine => Some(gen_stub(56, "OEMC_grpSetFontHelpLine")),
            WIPICGraphicsMethodId::GetFontHelpLine => Some(gen_stub(57, "OEMC_grpGetFontHelpLine")),
            WIPICGraphicsMethodId::EncodeImageEx => Some(gen_stub(58, "OEMC_grpEncodeImageEx")),
            WIPICGraphicsMethodId::GetImageInfo => Some(gen_stub(59, "OEMC_grpGetImageInfo")),
        },
        WIPICTableId::Interface3 => get_unk3_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Interface4 => {
            if function_id < 64 {
                Some(gen_stub(4, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Interface5 => {
            if function_id < 64 {
                Some(gen_stub(5, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Database => match WIPICDatabaseMethodId::try_from(function_id).ok()? {
            WIPICDatabaseMethodId::OpenDatabase => Some(database::open_database.into_body()),
            WIPICDatabaseMethodId::StreamRead => Some(database::stream_read.into_body()),
            WIPICDatabaseMethodId::StreamWrite => Some(database::stream_write.into_body()),
            WIPICDatabaseMethodId::CloseDatabase => Some(database::close_database.into_body()),
            WIPICDatabaseMethodId::SelectRecord => Some(database::select_record.into_body()),
            WIPICDatabaseMethodId::UpdateRecord => Some(gen_stub(5, "MC_dbUpdateRecord")),
            WIPICDatabaseMethodId::DeleteRecord => Some(database::delete_record.into_body()),
            WIPICDatabaseMethodId::ListRecord => Some(database::list_record.into_body()),
            WIPICDatabaseMethodId::SortRecords => Some(gen_stub(8, "MC_dbSortRecords")),
            WIPICDatabaseMethodId::GetAccessMode => Some(gen_stub(9, "MC_dbGetAccessMode")),
            WIPICDatabaseMethodId::GetNumberOfRecords => Some(gen_stub(10, "MC_dbGetNumberOfRecords")),
            WIPICDatabaseMethodId::GetRecordSize => Some(gen_stub(11, "MC_dbGetRecordSize")),
            WIPICDatabaseMethodId::ListDatabases => Some(gen_stub(12, "MC_dbListDataBases")),
            WIPICDatabaseMethodId::Unk13 => Some(gen_stub(13, "")),
            WIPICDatabaseMethodId::Unk14 => Some(gen_stub(14, "")),
            WIPICDatabaseMethodId::Unk15 => Some(gen_stub(15, "")),
            WIPICDatabaseMethodId::Unk16 => Some(database::unk16.into_body()),
        },
        WIPICTableId::Interface7 => {
            if function_id < 64 {
                Some(gen_stub(7, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Uic => get_uic_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Media => get_media_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Net => get_net_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Interface11 => {
            if function_id < 64 {
                Some(gen_stub(11, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Interface12 => get_unk12_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Interface13 => {
            if function_id < 64 {
                Some(gen_stub(13, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Interface14 => {
            if function_id < 64 {
                Some(gen_stub(14, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Interface15 => {
            if function_id < 64 {
                Some(gen_stub(15, "stub"))
            } else {
                None
            }
        }
        WIPICTableId::Interface16 => {
            if function_id < 64 {
                Some(gen_stub(16, "stub"))
            } else {
                None
            }
        }
    }
}
