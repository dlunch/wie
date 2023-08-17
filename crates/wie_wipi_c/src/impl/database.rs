use alloc::{vec, vec::Vec};

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
};

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented database{}", id)) };

    body.into_body()
}

pub fn get_database_method_table() -> Vec<CMethodBody> {
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
    ]
}
