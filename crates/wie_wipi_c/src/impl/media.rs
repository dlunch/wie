use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented media{}: {}", id, name)) };

    body.into_body()
}

pub fn get_media_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0, "MC_mdaClipCreate"),
        gen_stub(1, "MC_mdaClipFree"),
        gen_stub(2, "MC_mdaSetWaterMark"),
        gen_stub(3, "MC_mdaClipGetType"),
        gen_stub(4, "MC_mdaClipPutData"),
        gen_stub(5, "MC_mdaClipPutDataByFile"),
        gen_stub(6, "MC_mdaClipPutToneData"),
        gen_stub(7, "MC_mdaClipPutFreqToneData"),
        gen_stub(8, "MC_mdaClipGetData"),
        gen_stub(9, "MC_mdaClipAvailableDataSize"),
        gen_stub(10, "MC_mdaClipClearData"),
        gen_stub(11, "MC_mdaClipSetPosition"),
        gen_stub(12, "MC_mdaClipGetVolume"),
        gen_stub(13, "MC_mdaClipSetVolume"),
        gen_stub(14, "MC_mdaPlay"),
        gen_stub(15, "MC_mdaPause"),
        gen_stub(16, "MC_mdaResume"),
        gen_stub(17, "MC_mdaStop"),
        gen_stub(18, "MC_mdaRecord"),
        gen_stub(19, "MC_mdaGetVolume"),
        gen_stub(20, "MC_mdaSetVolume"),
        gen_stub(21, "MC_mdaVibrator"),
        gen_stub(22, "MC_mdaReserved1"),
        gen_stub(23, "MC_mdaReserved2"),
        gen_stub(24, "MC_mdaSetMuteState"),
        gen_stub(25, "MC_mdaGetMuteState"),
        gen_stub(26, "OEMC_mdaClipGetInfo"),
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
