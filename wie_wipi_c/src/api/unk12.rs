use alloc::{vec, vec::Vec};

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICResult, WIPICWord},
    method::MethodImpl,
};

async fn unk0(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk12_0()");

    Ok(0)
}

async fn unk1(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk12_1()");

    Ok(0)
}

async fn unk2(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk12_2()");

    Ok(0)
}

pub fn get_unk12_method_table() -> Vec<WIPICMethodBody> {
    vec![unk0.into_body(), unk1.into_body(), unk2.into_body()]
}
