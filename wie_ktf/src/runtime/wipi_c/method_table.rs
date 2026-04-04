use alloc::{format, vec, vec::Vec};

use wipi_types::{
    ktf::wipic::{WIPICDatabaseInterface, WIPICGraphicsInterface, WIPICKnlInterface},
    wipic::WIPICWord,
};

use wie_util::{Result, WieError};
use wie_wipi_c::{
    MethodImpl, WIPICContext, WIPICMethodBody,
    api::{database, graphics, kernel, media, misc, net, uic, util},
};

use crate::runtime::svc_ids::{WIPICDatabaseMethodId, WIPICGraphicsMethodId, WIPICKernelMethodId, WIPICTableId};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WieError::Unimplemented(format!("{id}: {name}"))) };

    body.into_body()
}

pub fn get_kernel_interface(context: &mut dyn WIPICContext) -> Result<WIPICKnlInterface> {
    let table_id = WIPICTableId::Kernel;

    Ok(WIPICKnlInterface {
        printk: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Printk))?,
        sprintk: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Sprintk))?,
        get_exec_names: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetExecNames))?,
        execute: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Execute))?,
        mexecute: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Mexecute))?,
        load: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Load))?,
        mload: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Mload))?,
        exit: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Exit))?,
        program_stop: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::ProgramStop))?,
        get_cur_program_id: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetCurProgramId))?,
        get_parent_program_id: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetParentProgramId))?,
        get_app_manager_id: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetAppManagerId))?,
        get_program_info: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetProgramInfo))?,
        get_access_level: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetAccessLevel))?,
        get_program_name: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetProgramName))?,
        create_shared_buf: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CreateSharedBuf))?,
        destroy_shared_buf: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::DestroySharedBuf))?,
        get_shared_buf: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetSharedBuf))?,
        get_shared_buf_size: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetSharedBufSize))?,
        resize_shared_buf: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::ResizeSharedBuf))?,
        alloc: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Alloc))?,
        calloc: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Calloc))?,
        free: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Free))?,
        get_total_memory: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetTotalMemory))?,
        get_free_memory: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetFreeMemory))?,
        def_timer: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::DefTimer))?,
        set_timer: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::SetTimer))?,
        unset_timer: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::UnsetTimer))?,
        current_time: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CurrentTime))?,
        get_system_property: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetSystemProperty))?,
        set_system_property: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::SetSystemProperty))?,
        get_resource_id: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetResourceId))?,
        get_resource: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetResource))?,
        reserved1: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved1))?,
        reserved2: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved2))?,
        reserved3: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved3))?,
        reserved4: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved4))?,
        reserved5: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved5))?,
        reserved6: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved6))?,
        reserved7: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved7))?,
        reserved8: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved8))?,
        reserved9: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved9))?,
        reserved10: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved10))?,
        reserved11: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved11))?,
        send_message: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::SendMessage))?,
        set_timer_ex: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::SetTimerEx))?,
        get_system_state: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetSystemState))?,
        create_system_progress_bar: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CreateSystemProgressBar))?,
        set_system_progress_bar: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::SetSystemProgressBar))?,
        destroy_system_progress_bar: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::DestroySystemProgressBar))?,
        execute_ex: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::ExecuteEx))?,
        get_proc_address: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetProcAddress))?,
        unload: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Unload))?,
        create_sys_message_box: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CreateSysMessageBox))?,
        destroy_sys_message_box: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::DestroySysMessageBox))?,
        get_program_id_list: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetProgramIdList))?,
        get_program_info2: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetProgramInfo2))?,
        reserved12: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved12))?,
        reserved13: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::Reserved13))?,
        create_app_private_area: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CreateAppPrivateArea))?,
        get_app_private_area: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetAppPrivateArea))?,
        create_lib_private_area: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::CreateLibPrivateArea))?,
        get_lib_private_area: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetLibPrivateArea))?,
        get_platform_version: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetPlatformVersion))?,
        get_token: context.make_svc_stub(table_id.function_id(WIPICKernelMethodId::GetToken))?,
    })
}

pub fn get_graphics_interface(context: &mut dyn WIPICContext) -> Result<WIPICGraphicsInterface> {
    let table_id = WIPICTableId::Graphics;

    Ok(WIPICGraphicsInterface {
        get_image_property: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetImageProperty))?,
        get_image_framebuffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetImageFramebuffer))?,
        get_screen_framebuffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetScreenFramebuffer))?,
        destroy_offscreen_framebuffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DestroyOffscreenFramebuffer))?,
        create_offscreen_framebuffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::CreateOffscreenFramebuffer))?,
        init_context: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::InitContext))?,
        set_context: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetContext))?,
        get_context: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetContext))?,
        put_pixel: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::PutPixel))?,
        draw_line: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawLine))?,
        draw_rect: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawRect))?,
        fill_rect: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::FillRect))?,
        copy_frame_buffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::CopyFrameBuffer))?,
        draw_image: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawImage))?,
        copy_area: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::CopyArea))?,
        draw_arc: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawArc))?,
        fill_arc: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::FillArc))?,
        draw_string: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawString))?,
        draw_unicode_string: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawUnicodeString))?,
        get_rgb_pixels: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetRgbPixels))?,
        set_rgb_pixels: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetRgbPixels))?,
        flush_lcd: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::FlushLcd))?,
        get_pixel_from_rgb: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetPixelFromRgb))?,
        get_rgb_from_pixel: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetRgbFromPixel))?,
        get_display_info: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetDisplayInfo))?,
        repaint: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::Repaint))?,
        get_font: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFont))?,
        get_font_height: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontHeight))?,
        get_font_ascent: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontAscent))?,
        get_font_descent: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontDescent))?,
        get_string_width: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetStringWidth))?,
        get_unicode_string_width: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetUnicodeStringWidth))?,
        create_image: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::CreateImage))?,
        destroy_image: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DestroyImage))?,
        decode_next_image: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DecodeNextImage))?,
        encode_image: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::EncodeImage))?,
        post_event: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::PostEvent))?,
        handle_input: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::HandleInput))?,
        set_current_mode: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetCurrentMode))?,
        get_current_mode: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetCurrentMode))?,
        get_support_mode_count: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetSupportModeCount))?,
        get_supported_modes: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetSupportedModes))?,
        fill_polygon: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::FillPolygon))?,
        draw_polygon: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::DrawPolygon))?,
        show_annunciator: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::ShowAnnunciator))?,
        get_annunciator_info: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetAnnunciatorInfo))?,
        set_annunciator_icon: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetAnnunciatorIcon))?,
        get_idle_help_line_info: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetIdleHelpLineInfo))?,
        show_help_line: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::ShowHelpLine))?,
        get_char_glyph: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetCharGlyph))?,
        create_image_ex: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::CreateImageEx))?,
        hide_help_line: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::HideHelpLine))?,
        set_clone_screen_framebuffer: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetCloneScreenFramebuffer))?,
        get_font_ex: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontEx))?,
        get_font_lists: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontLists))?,
        get_font_info: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontInfo))?,
        set_font_help_line: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::SetFontHelpLine))?,
        get_font_help_line: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetFontHelpLine))?,
        encode_image_ex: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::EncodeImageEx))?,
        get_image_info: context.make_svc_stub(table_id.function_id(WIPICGraphicsMethodId::GetImageInfo))?,
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

pub fn get_database_interface(context: &mut dyn WIPICContext) -> Result<WIPICDatabaseInterface> {
    let table_id = WIPICTableId::Database;

    Ok(WIPICDatabaseInterface {
        open_database: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::OpenDatabase))?,
        read_record_single: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::ReadRecordSingle))?,
        write_record_single: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::WriteRecordSingle))?,
        close_database: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::CloseDatabase))?,
        select_record: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::SelectRecord))?,
        update_record: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::UpdateRecord))?,
        delete_record: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::DeleteRecord))?,
        list_record: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::ListRecord))?,
        sort_records: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::SortRecords))?,
        get_access_mode: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::GetAccessMode))?,
        get_number_of_records: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::GetNumberOfRecords))?,
        get_record_size: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::GetRecordSize))?,
        list_databases: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::ListDatabases))?,
        unk13: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::Unk13))?,
        unk14: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::Unk14))?,
        unk15: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::Unk15))?,
        unk16: context.make_svc_stub(table_id.function_id(WIPICDatabaseMethodId::Unk16))?,
    })
}

pub fn get_uic_method_table() -> Vec<WIPICMethodBody> {
    vec![
        uic::create_application_context.into_body(),
        gen_stub(1, "MC_uicGetClass"),
        gen_stub(2, "MC_uicCreate"),
        gen_stub(3, "MC_uicDestroy"),
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
        gen_stub(26, "MC_uicGetMenuItem"),
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
        WIPICTableId::Kernel => match function_id {
            0 => Some(kernel::printk.into_body()),
            1 => Some(kernel::sprintk.into_body()),
            2 => Some(gen_stub(2, "MC_knlGetExecNames")),
            3 => Some(gen_stub(3, "MC_knlExecute")),
            4 => Some(gen_stub(4, "MC_knlMExecute")),
            5 => Some(gen_stub(5, "MC_knlLoad")),
            6 => Some(gen_stub(6, "MC_knlMLoad")),
            7 => Some(kernel::exit.into_body()),
            8 => Some(gen_stub(8, "MC_knlProgramStop")),
            9 => Some(kernel::get_cur_program_id.into_body()),
            10 => Some(gen_stub(10, "MC_knlGetParentProgramID")),
            11 => Some(gen_stub(11, "MC_knlGetAppManagerID")),
            12 => Some(gen_stub(12, "MC_knlGetProgramInfo")),
            13 => Some(gen_stub(13, "MC_knlGetAccessLevel")),
            14 => Some(kernel::get_program_name.into_body()),
            15 => Some(gen_stub(15, "MC_knlCreateSharedBuf")),
            16 => Some(gen_stub(16, "MC_knlDestroySharedBuf")),
            17 => Some(gen_stub(17, "MC_knlGetSharedBuf")),
            18 => Some(gen_stub(18, "MC_knlGetSharedBufSize")),
            19 => Some(gen_stub(19, "MC_knlResizeSharedBuf")),
            20 => Some(kernel::alloc.into_body()),
            21 => Some(kernel::calloc.into_body()),
            22 => Some(kernel::free.into_body()),
            23 => Some(kernel::get_total_memory.into_body()),
            24 => Some(kernel::get_free_memory.into_body()),
            25 => Some(kernel::def_timer.into_body()),
            26 => Some(kernel::set_timer.into_body()),
            27 => Some(kernel::unset_timer.into_body()),
            28 => Some(kernel::current_time.into_body()),
            29 => Some(kernel::get_system_property.into_body()),
            30 => Some(kernel::set_system_property.into_body()),
            31 => Some(kernel::get_resource_id.into_body()),
            32 => Some(kernel::get_resource.into_body()),
            34 => Some(gen_stub(34, "MC_knlReserved2")),
            35 => Some(gen_stub(35, "MC_knlReserved3")),
            36 => Some(gen_stub(36, "MC_knlReserved4")),
            37 => Some(gen_stub(37, "MC_knlReserved5")),
            38 => Some(gen_stub(38, "MC_knlReserved6")),
            39 => Some(gen_stub(39, "MC_knlReserved7")),
            40 => Some(gen_stub(40, "MC_knlReserved8")),
            41 => Some(gen_stub(41, "MC_knlReserved9")),
            42 => Some(gen_stub(42, "MC_knlReserved10")),
            43 => Some(gen_stub(43, "MC_knlReserved11")),
            44 => Some(gen_stub(44, "OEMC_knlSendMessage")),
            45 => Some(gen_stub(45, "OEMC_knlSetTimerEx")),
            46 => Some(gen_stub(46, "OEMC_knlGetSystemState")),
            47 => Some(gen_stub(47, "OEMC_knlCreateSystemProgressBar")),
            48 => Some(gen_stub(48, "OEMC_knlSetSystemProgressBar")),
            49 => Some(gen_stub(49, "OEMC_knlDestroySystemProgressBar")),
            50 => Some(gen_stub(50, "OEMC_knlExecuteEx")),
            51 => Some(gen_stub(51, "OEMC_knlGetProcAddress")),
            52 => Some(gen_stub(52, "OEMC_knlUnload")),
            53 => Some(gen_stub(53, "OEMC_knlCreateSysMessageBox")),
            54 => Some(gen_stub(54, "OEMC_knlDestroySysMessageBox")),
            55 => Some(gen_stub(55, "OEMC_knlGetProgramIDList")),
            56 => Some(gen_stub(56, "OEMC_knlGetProgramInfo")),
            57 => Some(gen_stub(57, "MC_knlReserved12")),
            58 => Some(gen_stub(58, "MC_knlReserved13")),
            59 => Some(gen_stub(59, "OEMC_knlCreateAppPrivateArea")),
            60 => Some(gen_stub(60, "OEMC_knlGetAppPrivateArea")),
            61 => Some(gen_stub(61, "OEMC_knlCreateLibPrivateArea")),
            62 => Some(gen_stub(62, "OEMC_knlGetLibPrivateArea")),
            63 => Some(gen_stub(63, "OEMC_knlGetPlatformVersion")),
            64 => Some(gen_stub(64, "OEMC_knlGetToken")),
            _ => None,
        },
        WIPICTableId::Util => get_util_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Misc => get_misc_method_table().into_iter().nth(function_id as usize),
        WIPICTableId::Graphics => match function_id {
            0 => Some(graphics::get_image_property.into_body()),
            1 => Some(gen_stub(1, "MC_grpGetImageFrameBuffer")),
            2 => Some(graphics::get_screen_framebuffer.into_body()),
            3 => Some(graphics::destroy_offscreen_framebuffer.into_body()),
            4 => Some(graphics::create_offscreen_framebuffer.into_body()),
            5 => Some(graphics::init_context.into_body()),
            6 => Some(graphics::set_context.into_body()),
            7 => Some(gen_stub(7, "MC_grpGetContext")),
            8 => Some(graphics::put_pixel.into_body()),
            9 => Some(graphics::draw_line.into_body()),
            10 => Some(graphics::draw_rect.into_body()),
            11 => Some(graphics::fill_rect.into_body()),
            12 => Some(graphics::copy_frame_buffer.into_body()),
            13 => Some(graphics::draw_image.into_body()),
            14 => Some(graphics::copy_area.into_body()),
            15 => Some(gen_stub(15, "MC_grpDrawArc")),
            16 => Some(gen_stub(16, "MC_grpFillArc")),
            17 => Some(graphics::draw_string.into_body()),
            18 => Some(gen_stub(18, "MC_grpDrawUnicodeString")),
            19 => Some(gen_stub(19, "MC_grpGetRGBPixels")),
            20 => Some(gen_stub(20, "MC_grpSetRGBPixels")),
            21 => Some(graphics::flush_lcd.into_body()),
            22 => Some(graphics::get_pixel_from_rgb.into_body()),
            23 => Some(graphics::get_rgb_from_pixel.into_body()),
            24 => Some(graphics::get_display_info.into_body()),
            25 => Some(graphics::repaint.into_body()),
            26 => Some(graphics::get_font.into_body()),
            27 => Some(graphics::get_font_height.into_body()),
            28 => Some(gen_stub(28, "MC_grpGetFontAscent")),
            29 => Some(gen_stub(29, "MC_grpGetFontDescent")),
            30 => Some(graphics::get_string_width.into_body()),
            31 => Some(gen_stub(31, "MC_grpGetUnicodeStringWidth")),
            32 => Some(graphics::create_image.into_body()),
            33 => Some(graphics::destroy_image.into_body()),
            34 => Some(gen_stub(34, "MC_grpDecodeNextImage")),
            35 => Some(gen_stub(35, "MC_grpEncodeImage")),
            36 => Some(graphics::post_event.into_body()),
            37 => Some(gen_stub(37, "MC_imHandleInput")),
            38 => Some(gen_stub(38, "MC_imSetCurrentMode")),
            39 => Some(gen_stub(39, "MC_imGetCurrentMode")),
            40 => Some(gen_stub(40, "MC_imGetSupportModeCount")),
            41 => Some(gen_stub(41, "MC_imGetSupportedModes")),
            42 => Some(gen_stub(42, "MC_grpFillPolygon")),
            43 => Some(gen_stub(43, "MC_grpDrawPolygon")),
            44 => Some(gen_stub(44, "OEMC_grpShowAnnunciator")),
            45 => Some(gen_stub(45, "OEMC_grpGetAnnunciatorInfo")),
            46 => Some(gen_stub(46, "OEMC_grp  SetAnnunciatorIcon")),
            47 => Some(gen_stub(47, "OEMC_grpGetIdleHelpLineInfo")),
            48 => Some(gen_stub(48, "OEMC_grpShowHelpLine")),
            49 => Some(gen_stub(49, "OEMC_grpGetCharGlyph")),
            50 => Some(gen_stub(50, "OEMC_grpCreateImageEx")),
            51 => Some(gen_stub(51, "OEMC_grpHideHelpLine")),
            52 => Some(gen_stub(52, "OEMC_grpSetCloneScreenFrameBuffer")),
            53 => Some(gen_stub(53, "OEMC_grpGetFontEx")),
            54 => Some(gen_stub(54, "OEMC_grpGetFontLists")),
            55 => Some(gen_stub(55, "OEMC_grpGetFontInfo")),
            56 => Some(gen_stub(56, "OEMC_grpSetFontHelpLine")),
            57 => Some(gen_stub(57, "OEMC_grpGetFontHelpLine")),
            58 => Some(gen_stub(58, "OEMC_grpEncodeImageEx")),
            59 => Some(gen_stub(59, "OEMC_grpGetImageInfo")),
            _ => None,
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
        WIPICTableId::Database => match function_id {
            0 => Some(database::open_database.into_body()),
            1 => Some(database::read_record_single.into_body()),
            2 => Some(database::write_record_single.into_body()),
            3 => Some(database::close_database.into_body()),
            4 => Some(database::select_record.into_body()),
            5 => Some(gen_stub(5, "MC_dbUpdateRecord")),
            6 => Some(database::delete_record.into_body()),
            7 => Some(database::list_record.into_body()),
            8 => Some(gen_stub(8, "MC_dbSortRecords")),
            9 => Some(gen_stub(9, "MC_dbGetAccessMode")),
            10 => Some(gen_stub(10, "MC_dbGetNumberOfRecords")),
            11 => Some(gen_stub(11, "MC_dbGetRecordSize")),
            12 => Some(gen_stub(12, "MC_dbListDataBases")),
            13 => Some(gen_stub(13, "")),
            14 => Some(gen_stub(14, "")),
            15 => Some(gen_stub(15, "")),
            16 => Some(database::unk16.into_body()),
            _ => None,
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
