use alloc::vec::Vec;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_base::util::write_generic;
use wie_core_arm::ArmCore;
use wie_impl_wipi_c::{
    r#impl::{
        database::get_database_method_table, graphics::get_graphics_method_table, kernel::get_kernel_method_table, media::get_media_method_table,
        misc::get_misc_method_table, net::get_net_method_table, stub::get_stub_method_table, uic::get_uic_method_table,
        unk12::get_unk12_method_table, unk3::get_unk3_method_table, util::get_util_method_table,
    },
    WIPICContext, WIPICMethodBody,
};

use crate::runtime::wipi_c::context::KtfWIPICContext;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WIPICInterface {
    interface_0: u32,
    interface_1: u32,
    interface_2: u32,
    interface_3: u32,
    interface_4: u32,
    interface_5: u32,
    interface_6: u32,
    interface_7: u32,
    interface_8: u32,
    interface_9: u32,
    interface_10: u32,
    interface_11: u32,
    interface_12: u32,
    interface_13: u32,
    interface_14: u32,
    interface_15: u32,
    interface_16: u32,
}

fn write_methods(context: &mut dyn WIPICContext, methods: Vec<WIPICMethodBody>) -> anyhow::Result<u32> {
    let address = context.alloc_raw((methods.len() * 4) as u32)?;

    let mut cursor = address;
    for method in methods {
        let address = context.register_function(method)?;

        write_generic(context, cursor, address)?;
        cursor += 4;
    }

    Ok(address)
}

pub fn get_wipic_knl_interface(core: &mut ArmCore, system: &mut System) -> anyhow::Result<u32> {
    let kernel_methods = get_kernel_method_table(get_wipic_interfaces);

    let mut context = KtfWIPICContext::new(core, system);
    let address = write_methods(&mut context, kernel_methods)?;

    Ok(address)
}

async fn get_wipic_interfaces(context: &mut dyn WIPICContext) -> anyhow::Result<u32> {
    tracing::trace!("get_wipic_interfaces");

    let interface_0 = write_methods(context, get_util_method_table())?;
    let interface_1 = write_methods(context, get_misc_method_table())?;
    let interface_2 = write_methods(context, get_graphics_method_table())?;
    let interface_3 = write_methods(context, get_unk3_method_table())?;
    let interface_4 = write_methods(context, get_stub_method_table(4))?;
    let interface_5 = write_methods(context, get_stub_method_table(5))?;
    let interface_6 = write_methods(context, get_database_method_table())?;
    let interface_7 = write_methods(context, get_stub_method_table(7))?;
    let interface_8 = write_methods(context, get_uic_method_table())?; // uic
    let interface_9 = write_methods(context, get_media_method_table())?;
    let interface_10 = write_methods(context, get_net_method_table())?;
    let interface_11 = write_methods(context, get_stub_method_table(11))?;
    let interface_12 = write_methods(context, get_unk12_method_table())?;
    let interface_13 = write_methods(context, get_stub_method_table(13))?;
    let interface_14 = write_methods(context, get_stub_method_table(14))?;
    let interface_15 = write_methods(context, get_stub_method_table(15))?;
    let interface_16 = write_methods(context, get_stub_method_table(16))?;

    let interface = WIPICInterface {
        interface_0,
        interface_1,
        interface_2,
        interface_3,
        interface_4,
        interface_5,
        interface_6,
        interface_7,
        interface_8,
        interface_9,
        interface_10,
        interface_11,
        interface_12,
        interface_13,
        interface_14,
        interface_15,
        interface_16,
    };

    let address = context.alloc_raw(size_of::<WIPICInterface>() as u32)?;

    write_generic(context, address, interface)?;

    Ok(address)
}
