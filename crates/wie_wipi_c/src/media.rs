use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
};

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move {
        log::warn!("media stub{} called", id);

        Ok(0)
    };

    body.into_body()
}

pub fn get_media_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0),
        gen_stub(1),
        gen_stub(2),
        gen_stub(3),
        gen_stub(4),
        gen_stub(5),
        gen_stub(6),
        gen_stub(7),
        gen_stub(8),
        gen_stub(9),
        gen_stub(10),
        gen_stub(11),
        gen_stub(12),
        gen_stub(13),
        gen_stub(14),
        gen_stub(15),
        gen_stub(16),
        gen_stub(17),
        gen_stub(18),
        gen_stub(19),
        gen_stub(20),
        gen_stub(21),
        gen_stub(22),
        gen_stub(23),
        gen_stub(24),
        gen_stub(25),
        gen_stub(26),
    ]
}
