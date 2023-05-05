use std::mem::size_of;

use crate::{
    core::arm::ArmCore,
    wipi::c::{get_graphics_method_table, get_kernel_method_table, Bridge, CError, CMethodBody, CResult},
};

use super::{java_bridge::get_wipi_jb_interface, Context};

#[repr(C)]
#[derive(Clone, Copy)]
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
}

pub fn get_interface(core: &mut ArmCore, context: &Context, r#struct: String) -> anyhow::Result<u32> {
    log::debug!("get_interface({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs()?);

            Ok(0)
        }
    }
}

pub struct CBridge<'a> {
    core: &'a mut ArmCore,
    context: &'a Context,
}

impl<'a> CBridge<'a> {
    pub fn new(core: &'a mut ArmCore, context: &'a Context) -> Self {
        Self { core, context }
    }
}

impl Bridge for CBridge<'_> {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        self.context.alloc(size)
    }

    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()> {
        self.core.write_raw(address, data)
    }

    fn register_function(&mut self, method: Box<dyn Fn(&mut dyn Bridge) -> CResult<u32>>) -> CResult<u32> {
        self.core.register_function(
            move |core: &mut ArmCore, context: &Context| {
                let mut bridge = CBridge::new(core, context);
                let result = method(&mut bridge)?;

                Ok::<_, anyhow::Error>(result)
            },
            self.context,
        )
    }
}

fn write_methods(bridge: &mut dyn Bridge, methods: Vec<Box<dyn CMethodBody<CError>>>) -> anyhow::Result<u32> {
    let address = bridge.alloc((methods.len() * 4) as u32)?;

    let mut cursor = address;
    for method in methods {
        let address = bridge.register_function(Box::new(move |bridge: &mut dyn Bridge| {
            let result = method.call(bridge, vec![])?;

            Ok::<_, anyhow::Error>(result)
        }))?;

        bridge.write_raw(cursor, &address.to_le_bytes())?;
        cursor += 4;
    }

    Ok(address)
}

fn get_wipic_knl_interface(core: &mut ArmCore, context: &Context) -> anyhow::Result<u32> {
    let kernel_methods = get_kernel_method_table(get_wipic_interfaces);

    let mut bridge = CBridge::new(core, context);
    let address = write_methods(&mut bridge, kernel_methods)?;

    Ok(address)
}

fn get_wipic_interfaces(bridge: &mut dyn Bridge) -> anyhow::Result<u32> {
    log::debug!("get_wipic_interfaces");

    let graphics_methods = get_graphics_method_table();
    let interface_2 = write_methods(bridge, graphics_methods)?;

    let interface = WIPICInterface {
        interface_0: 0,
        interface_1: 0,
        interface_2,
        interface_3: 0,
        interface_4: 0,
        interface_5: 0,
        interface_6: 0,
        interface_7: 0,
        interface_8: 0,
        interface_9: 0,
        interface_10: 0,
        interface_11: 0,
        interface_12: 0,
    };

    let address = bridge.alloc(size_of::<WIPICInterface>() as u32)?;

    let data = unsafe { std::slice::from_raw_parts(&interface as *const _ as *const u8, std::mem::size_of::<WIPICInterface>()) };
    bridge.write_raw(address, data)?;

    Ok(address)
}
