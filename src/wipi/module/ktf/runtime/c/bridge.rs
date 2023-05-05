use crate::{
    core::arm::ArmCore,
    wipi::c::{CBridge, CBridgeMethod, CResult},
};

use super::super::Context;

pub struct KtfCBridge<'a> {
    core: &'a mut ArmCore,
    context: &'a Context,
}

impl<'a> KtfCBridge<'a> {
    pub fn new(core: &'a mut ArmCore, context: &'a Context) -> Self {
        Self { core, context }
    }
}

impl CBridge for KtfCBridge<'_> {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        self.context.alloc(size)
    }

    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()> {
        self.core.write_raw(address, data)
    }

    fn register_function(&mut self, method: CBridgeMethod) -> CResult<u32> {
        self.core.register_function(
            move |core: &mut ArmCore, context: &Context| {
                let mut bridge = KtfCBridge::new(core, context);
                let result = method(&mut bridge)?;

                Ok::<_, anyhow::Error>(result)
            },
            self.context,
        )
    }
}
