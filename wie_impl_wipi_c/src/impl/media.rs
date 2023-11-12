use alloc::{string::String, vec, vec::Vec};
use core::mem::size_of;

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICResult, WIPICWord},
    method::MethodImpl,
};

#[repr(C)]
struct MdaClip {
    clip_id: i32,
    h_proc: i32,
    r#type: u8,
    in_use: bool,
    dev_id: i32,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
    mute: bool,
    watermark: i32,
    position: i32,
    quality: i32,
    mode: i32,
    state: i32,
    penpot: i32,
    num_slave: i32,

    clip_save: WIPICWord, // MC_MdaClip**

    audio_tone_saved_len: i32,
    audio_tone_len: i32,
    audio_tone: WIPICWord,          // MC_MdaToneType*
    audio_tone_duration: WIPICWord, // M_Int32 *

    audio_freq_saved_len: i32,
    audio_freq_len: i32,
    audio_hi_freq: WIPICWord,       // M_Int32 *
    audio_low_freq: WIPICWord,      // M_Int32 *
    audio_freq_duration: WIPICWord, // M_Int32 *

    sound_data_saved_len: i32,
    sound_data_len: i32,
    sound_data: WIPICWord, // M_Byte *

    original_volume: i32,

    pos: i8,
    codec_config_data_size: i32,
    codec_config_data: WIPICWord, // M_Byte *
    tick_duration: i32,

    b_control: bool,

    movie_record_size_width: i32,
    movie_record_size_height: i32,
    max_record_length: i32,

    temp_record_space: WIPICWord, // M_Byte *
    temp_record_space_size: i32,
    temp_reord_size: i32,

    next_ptr: WIPICWord, // MC_MdaClip*

    mda_id: i32,
    device_info: i32,
}

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented media{}: {}", id, name)) };

    body.into_body()
}

async fn clip_create(context: &mut dyn WIPICContext, r#type: String, buf_size: WIPICWord, callback: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaClipCreate({}, {:#x}, {:#x})", r#type, buf_size, callback);

    let clip = context.alloc_raw(size_of::<MdaClip>() as u32)?;

    Ok(clip)
}

async fn clip_get_type(_context: &mut dyn WIPICContext, clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaClipGetType({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    Ok(0)
}

async fn get_mute_state(_context: &mut dyn WIPICContext, source: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaGetMuteState({:#x})", source);

    Ok(0)
}

async fn clip_get_info(
    _context: &mut dyn WIPICContext,
    clip: WIPICWord,
    command: WIPICWord,
    buf: WIPICWord,
    buf_size: WIPICWord,
) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub OEMC_mdaClipGetInfo({:#x}, {:#x}, {:#x}, {:#x})", clip, command, buf, buf_size);

    Ok(0)
}

async fn clip_put_data(context: &mut dyn WIPICContext, clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaClipPutData({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    let data = context.read_bytes(buf, buf_size)?;

    context.backend().audio().load_mmf(&data)?;

    Ok(0)
}

async fn clip_get_data(_context: &mut dyn WIPICContext, clip: WIPICWord, buf: WIPICWord, buf_size: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaClipGetData({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    Ok(0)
}

async fn clip_set_position(_context: &mut dyn WIPICContext, clip: WIPICWord, ms: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaClipSetPosition({:#x}, {:#x})", clip, ms);

    Ok(0)
}

async fn play(_context: &mut dyn WIPICContext, clip: WIPICWord, repeat: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaPlay({:#x}, {})", clip, repeat);

    Ok(0)
}

async fn pause(_context: &mut dyn WIPICContext, clip: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaPause({:#x})", clip);

    Ok(0)
}

async fn resume(_context: &mut dyn WIPICContext, clip: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaResume({:#x})", clip);

    Ok(0)
}

async fn stop(_context: &mut dyn WIPICContext, clip: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaStop({:#x})", clip);

    Ok(0)
}

async fn record(_context: &mut dyn WIPICContext, clip: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub MC_mdaRecord({:#x})", clip);

    Ok(0)
}

pub fn get_media_method_table() -> Vec<WIPICMethodBody> {
    vec![
        clip_create.into_body(),
        gen_stub(1, "MC_mdaClipFree"),
        gen_stub(2, "MC_mdaSetWaterMark"),
        clip_get_type.into_body(),
        clip_put_data.into_body(),
        gen_stub(5, "MC_mdaClipPutDataByFile"),
        gen_stub(6, "MC_mdaClipPutToneData"),
        gen_stub(7, "MC_mdaClipPutFreqToneData"),
        clip_get_data.into_body(),
        gen_stub(9, "MC_mdaClipAvailableDataSize"),
        gen_stub(10, "MC_mdaClipClearData"),
        clip_set_position.into_body(),
        gen_stub(12, "MC_mdaClipGetVolume"),
        gen_stub(13, "MC_mdaClipSetVolume"),
        play.into_body(),
        pause.into_body(),
        resume.into_body(),
        stop.into_body(),
        record.into_body(),
        gen_stub(19, "MC_mdaGetVolume"),
        gen_stub(20, "MC_mdaSetVolume"),
        gen_stub(21, "MC_mdaVibrator"),
        gen_stub(22, "MC_mdaReserved1"),
        gen_stub(23, "MC_mdaReserved2"),
        gen_stub(24, "MC_mdaSetMuteState"),
        get_mute_state.into_body(),
        clip_get_info.into_body(),
        // gen_stub(27, "OEMC_mdaClipControl"),
        // gen_stub(28, "OEMC_mdaSetClipArea"),
        // gen_stub(29, "OEMC_mdaReleaseClipArea"),
        // gen_stub(30, "OEMC_mdaUpdateClipArea"),
        // gen_stub(31, "OEMC_mdaGetDefaultVolume"),
        // gen_stub(32, "OEMC_mdaSetDefaultVolume"),
        // gen_stub(33, "MC_mdaReserved3"),
        // gen_stub(34, "MC_mdaReserved4"),
        // gen_stub(35, "OEMC_mdaClipGetPosition"),
        // gen_stub(36, "MC_mdaReserved5"),
        // gen_stub(37, "MC_mdaReserved6"),
        // gen_stub(38, "OEMC_mdaGetInfo"),
        // gen_stub(39, "OEMC_mdaClipPutDataEx"),
    ]
}
