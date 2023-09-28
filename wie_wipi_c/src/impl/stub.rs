use alloc::vec::Vec;

use crate::{
    base::{CContext, CMethodBody, CWord},
    method::MethodImpl,
};

fn gen_stub(interface: CWord, id: CWord) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented unknown{}_{}", interface, id)) };

    body.into_body()
}

pub fn get_stub_method_table(interface: CWord) -> Vec<CMethodBody> {
    (0..64).map(|x| gen_stub(interface, x)).collect::<Vec<_>>()
}
