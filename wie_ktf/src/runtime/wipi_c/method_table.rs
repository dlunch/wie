use alloc::{format, vec, vec::Vec};

use wipi_types::{
    ktf::wipic::{WIPICGraphicsInterface, WIPICKnlInterface},
    wipic::WIPICWord,
};

use wie_util::{Result, WieError};
use wie_wipi_c::{
    MethodImpl, WIPICContext, WIPICMethodBody,
    api::{database, graphics, kernel, media, misc, net, uic, util},
};

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WieError::Unimplemented(format!("{id}: {name}"))) };

    body.into_body()
}

pub fn get_kernel_interface<M, F, R, P>(context: &mut dyn WIPICContext, reserved1: M) -> Result<WIPICKnlInterface>
where
    M: MethodImpl<F, R, WieError, P>,
{
    Ok(WIPICKnlInterface {
        printk: context.register_function(kernel::printk.into_body())?,
        sprintk: context.register_function(kernel::sprintk.into_body())?,
        get_exec_names: context.register_function(gen_stub(2, "MC_knlGetExecNames"))?,
        execute: context.register_function(gen_stub(3, "MC_knlExecute"))?,
        mexecute: context.register_function(gen_stub(4, "MC_knlMExecute"))?,
        load: context.register_function(gen_stub(5, "MC_knlLoad"))?,
        mload: context.register_function(gen_stub(6, "MC_knlMLoad"))?,
        exit: context.register_function(kernel::exit.into_body())?,
        program_stop: context.register_function(gen_stub(8, "MC_knlProgramStop"))?,
        get_cur_program_id: context.register_function(kernel::get_cur_program_id.into_body())?,
        get_parent_program_id: context.register_function(gen_stub(10, "MC_knlGetParentProgramID"))?,
        get_app_manager_id: context.register_function(gen_stub(11, "MC_knlGetAppManagerID"))?,
        get_program_info: context.register_function(gen_stub(12, "MC_knlGetProgramInfo"))?,
        get_access_level: context.register_function(gen_stub(13, "MC_knlGetAccessLevel"))?,
        get_program_name: context.register_function(kernel::get_program_name.into_body())?,
        create_shared_buf: context.register_function(gen_stub(15, "MC_knlCreateSharedBuf"))?,
        destroy_shared_buf: context.register_function(gen_stub(16, "MC_knlDestroySharedBuf"))?,
        get_shared_buf: context.register_function(gen_stub(17, "MC_knlGetSharedBuf"))?,
        get_shared_buf_size: context.register_function(gen_stub(18, "MC_knlGetSharedBufSize"))?,
        resize_shared_buf: context.register_function(gen_stub(19, "MC_knlResizeSharedBuf"))?,
        alloc: context.register_function(kernel::alloc.into_body())?,
        calloc: context.register_function(kernel::calloc.into_body())?,
        free: context.register_function(kernel::free.into_body())?,
        get_total_memory: context.register_function(kernel::get_total_memory.into_body())?,
        get_free_memory: context.register_function(kernel::get_free_memory.into_body())?,
        def_timer: context.register_function(kernel::def_timer.into_body())?,
        set_timer: context.register_function(kernel::set_timer.into_body())?,
        unset_timer: context.register_function(kernel::unset_timer.into_body())?,
        current_time: context.register_function(kernel::current_time.into_body())?,
        get_system_property: context.register_function(kernel::get_system_property.into_body())?,
        set_system_property: context.register_function(kernel::set_system_property.into_body())?,
        get_resource_id: context.register_function(kernel::get_resource_id.into_body())?,
        get_resource: context.register_function(kernel::get_resource.into_body())?,
        reserved1: context.register_function(reserved1.into_body())?,
        reserved2: context.register_function(gen_stub(34, "MC_knlReserved2"))?,
        reserved3: context.register_function(gen_stub(35, "MC_knlReserved3"))?,
        reserved4: context.register_function(gen_stub(36, "MC_knlReserved4"))?,
        reserved5: context.register_function(gen_stub(37, "MC_knlReserved5"))?,
        reserved6: context.register_function(gen_stub(38, "MC_knlReserved6"))?,
        reserved7: context.register_function(gen_stub(39, "MC_knlReserved7"))?,
        reserved8: context.register_function(gen_stub(40, "MC_knlReserved8"))?,
        reserved9: context.register_function(gen_stub(41, "MC_knlReserved9"))?,
        reserved10: context.register_function(gen_stub(42, "MC_knlReserved10"))?,
        reserved11: context.register_function(gen_stub(43, "MC_knlReserved11"))?,
        send_message: context.register_function(gen_stub(44, "OEMC_knlSendMessage"))?,
        set_timer_ex: context.register_function(gen_stub(45, "OEMC_knlSetTimerEx"))?,
        get_system_state: context.register_function(gen_stub(46, "OEMC_knlGetSystemState"))?,
        create_system_progress_bar: context.register_function(gen_stub(47, "OEMC_knlCreateSystemProgressBar"))?,
        set_system_progress_bar: context.register_function(gen_stub(48, "OEMC_knlSetSystemProgressBar"))?,
        destroy_system_progress_bar: context.register_function(gen_stub(49, "OEMC_knlDestroySystemProgressBar"))?,
        execute_ex: context.register_function(gen_stub(50, "OEMC_knlExecuteEx"))?,
        get_proc_address: context.register_function(gen_stub(51, "OEMC_knlGetProcAddress"))?,
        unload: context.register_function(gen_stub(52, "OEMC_knlUnload"))?,
        create_sys_message_box: context.register_function(gen_stub(53, "OEMC_knlCreateSysMessageBox"))?,
        destroy_sys_message_box: context.register_function(gen_stub(54, "OEMC_knlDestroySysMessageBox"))?,
        get_program_id_list: context.register_function(gen_stub(55, "OEMC_knlGetProgramIDList"))?,
        get_program_info2: context.register_function(gen_stub(56, "OEMC_knlGetProgramInfo"))?,
        reserved12: context.register_function(gen_stub(57, "MC_knlReserved12"))?,
        reserved13: context.register_function(gen_stub(58, "MC_knlReserved13"))?,
        create_app_private_area: context.register_function(gen_stub(59, "OEMC_knlCreateAppPrivateArea"))?,
        get_app_private_area: context.register_function(gen_stub(60, "OEMC_knlGetAppPrivateArea"))?,
        create_lib_private_area: context.register_function(gen_stub(61, "OEMC_knlCreateLibPrivateArea"))?,
        get_lib_private_area: context.register_function(gen_stub(62, "OEMC_knlGetLibPrivateArea"))?,
        get_platform_version: context.register_function(gen_stub(63, "OEMC_knlGetPlatformVersion"))?,
        get_token: context.register_function(gen_stub(64, "OEMC_knlGetToken"))?,
    })
}

pub fn get_graphics_interface(context: &mut dyn WIPICContext) -> Result<WIPICGraphicsInterface> {
    Ok(WIPICGraphicsInterface {
        get_image_property: context.register_function(graphics::get_image_property.into_body())?,
        get_image_framebuffer: context.register_function(gen_stub(1, "MC_grpGetImageFrameBuffer"))?,
        get_screen_framebuffer: context.register_function(graphics::get_screen_framebuffer.into_body())?,
        destroy_offscreen_framebuffer: context.register_function(graphics::destroy_offscreen_framebuffer.into_body())?,
        create_offscreen_framebuffer: context.register_function(graphics::create_offscreen_framebuffer.into_body())?,
        init_context: context.register_function(graphics::init_context.into_body())?,
        set_context: context.register_function(graphics::set_context.into_body())?,
        get_context: context.register_function(gen_stub(7, "MC_grpGetContext"))?,
        put_pixel: context.register_function(graphics::put_pixel.into_body())?,
        draw_line: context.register_function(graphics::draw_line.into_body())?,
        draw_rect: context.register_function(graphics::draw_rect.into_body())?,
        fill_rect: context.register_function(graphics::fill_rect.into_body())?,
        copy_frame_buffer: context.register_function(graphics::copy_frame_buffer.into_body())?,
        draw_image: context.register_function(graphics::draw_image.into_body())?,
        copy_area: context.register_function(graphics::copy_area.into_body())?,
        draw_arc: context.register_function(gen_stub(15, "MC_grpDrawArc"))?,
        fill_arc: context.register_function(gen_stub(16, "MC_grpFillArc"))?,
        draw_string: context.register_function(graphics::draw_string.into_body())?,
        draw_unicode_string: context.register_function(gen_stub(18, "MC_grpDrawUnicodeString"))?,
        get_rgb_pixels: context.register_function(gen_stub(19, "MC_grpGetRGBPixels"))?,
        set_rgb_pixels: context.register_function(gen_stub(20, "MC_grpSetRGBPixels"))?,
        flush_lcd: context.register_function(graphics::flush_lcd.into_body())?,
        get_pixel_from_rgb: context.register_function(graphics::get_pixel_from_rgb.into_body())?,
        get_rgb_from_pixel: context.register_function(graphics::get_rgb_from_pixel.into_body())?,
        get_display_info: context.register_function(graphics::get_display_info.into_body())?,
        repaint: context.register_function(graphics::repaint.into_body())?,
        get_font: context.register_function(graphics::get_font.into_body())?,
        get_font_height: context.register_function(graphics::get_font_height.into_body())?,
        get_font_ascent: context.register_function(gen_stub(28, "MC_grpGetFontAscent"))?,
        get_font_descent: context.register_function(gen_stub(29, "MC_grpGetFontDescent"))?,
        get_string_width: context.register_function(graphics::get_string_width.into_body())?,
        get_unicode_string_width: context.register_function(gen_stub(31, "MC_grpGetUnicodeStringWidth"))?,
        create_image: context.register_function(graphics::create_image.into_body())?,
        destroy_image: context.register_function(graphics::destroy_image.into_body())?,
        decode_next_image: context.register_function(gen_stub(34, "MC_grpDecodeNextImage"))?,
        encode_image: context.register_function(gen_stub(35, "MC_grpEncodeImage"))?,
        post_event: context.register_function(graphics::post_event.into_body())?,
        handle_input: context.register_function(gen_stub(37, "MC_imHandleInput"))?,
        set_current_mode: context.register_function(gen_stub(38, "MC_imSetCurrentMode"))?,
        get_current_mode: context.register_function(gen_stub(39, "MC_imGetCurrentMode"))?,
        get_support_mode_count: context.register_function(gen_stub(40, "MC_imGetSupportModeCount"))?,
        get_supported_modes: context.register_function(gen_stub(41, "MC_imGetSupportedModes"))?,
        fill_polygon: context.register_function(gen_stub(42, "MC_grpFillPolygon"))?,
        draw_polygon: context.register_function(gen_stub(43, "MC_grpDrawPolygon"))?,
        show_annunciator: context.register_function(gen_stub(44, "OEMC_grpShowAnnunciator"))?,
        get_annunciator_info: context.register_function(gen_stub(45, "OEMC_grpGetAnnunciatorInfo"))?,
        set_annunciator_icon: context.register_function(gen_stub(46, "OEMC_grp  SetAnnunciatorIcon"))?,
        get_idle_help_line_info: context.register_function(gen_stub(47, "OEMC_grpGetIdleHelpLineInfo"))?,
        show_help_line: context.register_function(gen_stub(48, "OEMC_grpShowHelpLine"))?,
        get_char_glyph: context.register_function(gen_stub(49, "OEMC_grpGetCharGlyph"))?,
        create_image_ex: context.register_function(gen_stub(50, "OEMC_grpCreateImageEx"))?,
        hide_help_line: context.register_function(gen_stub(51, "OEMC_grpHideHelpLine"))?,
        set_clone_screen_framebuffer: context.register_function(gen_stub(52, "OEMC_grpSetCloneScreenFrameBuffer"))?,
        get_font_ex: context.register_function(gen_stub(53, "OEMC_grpGetFontEx"))?,
        get_font_lists: context.register_function(gen_stub(54, "OEMC_grpGetFontLists"))?,
        get_font_info: context.register_function(gen_stub(55, "OEMC_grpGetFontInfo"))?,
        set_font_help_line: context.register_function(gen_stub(56, "OEMC_grpSetFontHelpLine"))?,
        get_font_help_line: context.register_function(gen_stub(57, "OEMC_grpGetFontHelpLine"))?,
        encode_image_ex: context.register_function(gen_stub(58, "OEMC_grpEncodeImageEx"))?,
        get_image_info: context.register_function(gen_stub(59, "OEMC_grpGetImageInfo"))?,
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

pub fn get_database_method_table() -> Vec<WIPICMethodBody> {
    vec![
        database::open_database.into_body(),
        database::read_record_single.into_body(),
        database::write_record_single.into_body(),
        database::close_database.into_body(),
        database::select_record.into_body(),
        gen_stub(5, "MC_dbUpdateRecord"),
        database::delete_record.into_body(),
        database::list_record.into_body(),
        gen_stub(8, "MC_dbSortRecords"),
        gen_stub(9, "MC_dbGetAccessMode"),
        gen_stub(10, "MC_dbGetNumberOfRecords"),
        gen_stub(11, "MC_dbGetRecordSize"),
        gen_stub(12, "MC_dbListDataBases"),
        gen_stub(13, ""),
        gen_stub(14, ""),
        gen_stub(15, ""),
        database::unk16.into_body(),
    ]
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
        tracing::warn!("stub unk{}-{}", id, index);
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
