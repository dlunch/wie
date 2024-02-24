use alloc::{vec, vec::Vec};

use crate::{context::WIPICContext, method::MethodImpl, WIPICMethodBody, WIPICResult, WIPICWord};

async fn unk0(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk3_0()");

    Ok(0)
}

async fn unk1(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk3_1()");

    Ok(0)
}

async fn unk2(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk3_2()");

    Ok(0)
}

async fn unk3(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk3_3()");

    Ok(0)
}

async fn unk4(_context: &mut dyn WIPICContext) -> WIPICResult<WIPICWord> {
    tracing::warn!("stub unk3_4()");

    Ok(0)
}

pub fn get_unk3_method_table() -> Vec<WIPICMethodBody> {
    vec![unk0.into_body(), unk1.into_body(), unk2.into_body(), unk3.into_body(), unk4.into_body()]
}
