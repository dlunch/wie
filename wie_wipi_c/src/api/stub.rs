use alloc::{format, vec::Vec};

use crate::{context::WIPICContext, method::MethodImpl, WIPICError, WIPICMethodBody, WIPICWord};

fn gen_stub(interface: WIPICWord, id: WIPICWord) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WIPICError::Unimplemented(format!("unknown{}_{}", interface, id))) };

    body.into_body()
}

pub fn get_stub_method_table(interface: WIPICWord) -> Vec<WIPICMethodBody> {
    (0..64).map(|x| gen_stub(interface, x)).collect::<Vec<_>>()
}
