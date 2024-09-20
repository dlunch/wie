use alloc::vec::Vec;
use core::mem::size_of;
use jvm::Jvm;

use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_core_arm::ArmCore;
use wie_util::{write_generic, Result};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

use crate::runtime::wipi_c::{context::KtfWIPICContext, method_table};

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

fn write_methods(context: &mut dyn WIPICContext, methods: Vec<WIPICMethodBody>) -> Result<u32> {
    let address = context.alloc_raw((methods.len() * 4) as u32)?;

    let mut cursor = address;
    for method in methods {
        let address = context.register_function(method)?;

        write_generic(context, cursor, address)?;
        cursor += 4;
    }

    Ok(address)
}

pub fn get_wipic_knl_interface(core: &mut ArmCore, system: &mut System, jvm: &Jvm) -> Result<u32> {
    let kernel_methods = method_table::get_kernel_method_table(get_wipic_interfaces);

    let mut context = KtfWIPICContext::new(core.clone(), system.clone(), jvm.clone());
    let address = write_methods(&mut context, kernel_methods).unwrap();

    Ok(address)
}

pub async fn get_wipic_interfaces(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::trace!("get_wipic_interfaces");

    let interface_0 = write_methods(context, method_table::get_util_method_table())?;
    let interface_1 = write_methods(context, method_table::get_misc_method_table())?;
    let interface_2 = write_methods(context, method_table::get_graphics_method_table())?;
    let interface_3 = write_methods(context, method_table::get_unk3_method_table())?;
    let interface_4 = write_methods(context, method_table::get_stub_method_table(4))?;
    let interface_5 = write_methods(context, method_table::get_stub_method_table(5))?;
    let interface_6 = write_methods(context, method_table::get_database_method_table())?;
    let interface_7 = write_methods(context, method_table::get_stub_method_table(7))?;
    let interface_8 = write_methods(context, method_table::get_uic_method_table())?;
    let interface_9 = write_methods(context, method_table::get_media_method_table())?;
    let interface_10 = write_methods(context, method_table::get_net_method_table())?;
    let interface_11 = write_methods(context, method_table::get_stub_method_table(11))?;
    let interface_12 = write_methods(context, method_table::get_unk12_method_table())?;
    let interface_13 = write_methods(context, method_table::get_stub_method_table(13))?;
    let interface_14 = write_methods(context, method_table::get_stub_method_table(14))?;
    let interface_15 = write_methods(context, method_table::get_stub_method_table(15))?;
    let interface_16 = write_methods(context, method_table::get_stub_method_table(16))?;

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
