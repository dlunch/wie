use alloc::vec::Vec;

use wie_backend::Backend;
use wie_base::util::{ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_c::{CContextBase, CContextMethod, CResult};

pub struct KtfCContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
}

impl<'a> KtfCContext<'a> {
    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        Self { core, backend }
    }
}

impl CContextBase for KtfCContext<'_> {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        Allocator::alloc(self.core, size)
    }

    fn register_function(&mut self, method: CContextMethod) -> CResult<u32> {
        self.core.register_function(
            move |core: &mut ArmCore, backend: &mut Backend, a0: u32, a1: u32, a2: u32| {
                let mut context = KtfCContext::new(core, backend);

                // Hack to put lifetime on context.
                let context: &mut KtfCContext<'static> = unsafe { core::mem::transmute(&mut context) };

                let result = method(context, &[a0, a1, a2])?;

                Ok::<_, anyhow::Error>(result)
            },
            self.backend,
        )
    }
    fn backend(&mut self) -> &mut Backend {
        self.backend
    }
}

impl ByteRead for KtfCContext<'_> {
    fn read_bytes(&self, address: u32, size: u32) -> anyhow::Result<Vec<u8>> {
        self.core.read_bytes(address, size)
    }
}

impl ByteWrite for KtfCContext<'_> {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()> {
        self.core.write_bytes(address, data)
    }
}
