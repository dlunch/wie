use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody, CResult},
    method::MethodImpl,
};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented media{}: {}", id, name)) };

    body.into_body()
}

async fn clip_create(_context: &mut dyn CContext, r#type: u32, buf_size: u32, callback: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaClipCreate({:#x}, {:#x}, {:#x})", r#type, buf_size, callback);

    Ok(0)
}

async fn clip_get_type(_context: &mut dyn CContext, clip: u32, buf: u32, buf_size: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaClipGetType({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    Ok(0)
}

async fn get_mute_state(_context: &mut dyn CContext, source: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaGetMuteState({:#x})", source);

    Ok(0)
}

async fn clip_get_info(_context: &mut dyn CContext, clip: u32, command: u32, buf: u32, buf_size: u32) -> CResult<u32> {
    tracing::warn!("stub OEMC_mdaClipGetInfo({:#x}, {:#x}, {:#x}, {:#x})", clip, command, buf, buf_size);

    Ok(0)
}

async fn clip_put_data(_context: &mut dyn CContext, clip: u32, buf: u32, buf_size: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaClipPutData({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    Ok(0)
}

async fn clip_get_data(_context: &mut dyn CContext, clip: u32, buf: u32, buf_size: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaClipGetData({:#x}, {:#x}, {:#x})", clip, buf, buf_size);

    Ok(0)
}

async fn clip_set_position(_context: &mut dyn CContext, clip: u32, ms: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaClipSetPosition({:#x}, {:#x})", clip, ms);

    Ok(0)
}

async fn stop(_context: &mut dyn CContext, clip: u32) -> CResult<u32> {
    tracing::warn!("stub MC_mdaStop({:#x})", clip);

    Ok(0)
}

pub fn get_media_method_table() -> Vec<CMethodBody> {
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
        gen_stub(14, "MC_mdaPlay"),
        gen_stub(15, "MC_mdaPause"),
        gen_stub(16, "MC_mdaResume"),
        stop.into_body(),
        gen_stub(18, "MC_mdaRecord"),
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
