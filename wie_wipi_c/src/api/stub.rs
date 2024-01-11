use alloc::vec::Vec;

use crate::{
    base::{WIPICContext, WIPICMethodBody, WIPICWord},
    method::MethodImpl,
};

fn gen_stub(interface: WIPICWord, id: WIPICWord) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented unknown{}_{}", interface, id)) };

    body.into_body()
}

pub fn get_stub_method_table(interface: WIPICWord) -> Vec<WIPICMethodBody> {
    (0..64).map(|x| gen_stub(interface, x)).collect::<Vec<_>>()
}
