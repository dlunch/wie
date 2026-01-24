use alloc::vec::Vec;
use core::mem::{size_of, size_of_val};

use bytemuck::Pod;

use jvm::Jvm;
use wipi_types::ktf::wipic::WIPICInterface;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

use crate::runtime::wipi_c::{
    context::KtfWIPICContext,
    method_table::{self, get_database_interface, get_graphics_interface},
};

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
    let mut context = KtfWIPICContext::new(core.clone(), system.clone(), jvm.clone());
    let kernel_interface = method_table::get_kernel_interface(&mut context, get_wipic_interfaces)?;

    let address = Allocator::alloc(core, size_of_val(&kernel_interface) as u32)?;
    write_generic(core, address, kernel_interface)?;

    Ok(address)
}

fn write_interface<T: Pod>(context: &mut dyn WIPICContext, interface: T) -> Result<u32> {
    let address = context.alloc_raw(size_of_val(&interface) as u32)?;
    write_generic(context, address, interface)?;

    Ok(address)
}

pub async fn get_wipic_interfaces(context: &mut dyn WIPICContext) -> Result<u32> {
    tracing::trace!("get_wipic_interfaces");

    let graphics_interface = get_graphics_interface(context)?;
    let database_interface = get_database_interface(context)?;

    let util_interface = write_methods(context, method_table::get_util_method_table())?;
    let misc_interface = write_methods(context, method_table::get_misc_method_table())?;
    let graphics_interface = write_interface(context, graphics_interface)?;
    let interface_3 = write_methods(context, method_table::get_unk3_method_table())?;
    let interface_4 = write_methods(context, method_table::get_stub_method_table(4))?;
    let interface_5 = write_methods(context, method_table::get_stub_method_table(5))?;
    let database_interface = write_interface(context, database_interface)?;
    let interface_7 = write_methods(context, method_table::get_stub_method_table(7))?;
    let uic_interface = write_methods(context, method_table::get_uic_method_table())?;
    let media_interface = write_methods(context, method_table::get_media_method_table())?;
    let net_interface = write_methods(context, method_table::get_net_method_table())?;
    let interface_11 = write_methods(context, method_table::get_stub_method_table(11))?;
    let interface_12 = write_methods(context, method_table::get_unk12_method_table())?;
    let interface_13 = write_methods(context, method_table::get_stub_method_table(13))?;
    let interface_14 = write_methods(context, method_table::get_stub_method_table(14))?;
    let interface_15 = write_methods(context, method_table::get_stub_method_table(15))?;
    let interface_16 = write_methods(context, method_table::get_stub_method_table(16))?;

    let interface = WIPICInterface {
        util_interface,
        misc_interface,
        graphics_interface,
        interface_3,
        interface_4,
        interface_5,
        database_interface,
        interface_7,
        uic_interface,
        media_interface,
        net_interface,
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
