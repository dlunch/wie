use crate::{
    backend::Backend,
    core::arm::{allocator::Allocator, ArmCore},
    wipi::c::{CBridge, CBridgeMethod, CResult},
};

pub struct KtfCBridge {
    core: ArmCore,
    backend: Backend,
}

impl KtfCBridge {
    pub fn new(core: ArmCore, backend: Backend) -> Self {
        Self { core, backend }
    }
}

impl CBridge for KtfCBridge {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        Allocator::alloc(&mut self.core, size)
    }

    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()> {
        self.core.write_raw(address, data)
    }

    fn register_function(&mut self, method: CBridgeMethod) -> CResult<u32> {
        self.core.register_function(
            move |core: ArmCore, backend: Backend, a0: u32, a1: u32, a2: u32| {
                let mut context = KtfCBridge::new(core, backend);

                let result = method(&mut context, vec![a0, a1, a2])?;

                Ok::<_, anyhow::Error>(result)
            },
            &self.backend,
        )
    }
    fn backend(&mut self) -> &mut Backend {
        &mut self.backend
    }
}
