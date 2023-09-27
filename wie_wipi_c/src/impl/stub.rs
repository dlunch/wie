use alloc::vec::Vec;

use crate::{
    base::{CContext, CMethodBody},
    method::MethodImpl,
};

fn gen_stub(interface: u32, id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented unknown{}_{}", interface, id)) };

    body.into_body()
}

pub fn get_stub_method_table(interface: u32) -> Vec<CMethodBody> {
    (0..64).map(|x| gen_stub(interface, x)).collect::<Vec<_>>()
}
