use alloc::{boxed::Box, vec::Vec};

use wie_backend::Backend;
use wie_base::util::{ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, ArmCoreError, EmulatedFunction, EmulatedFunctionParam};
use wie_wipi_c::{CContext, CMethodBody, CResult};

pub struct KtfCContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
}

impl<'a> KtfCContext<'a> {
    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        Self { core, backend }
    }
}

impl CContext for KtfCContext<'_> {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        Allocator::alloc(self.core, size)
    }

    fn register_function(&mut self, body: CMethodBody) -> CResult<u32> {
        let proxy = CMethodProxy::new(body);

        self.core.register_function(proxy, self.backend)
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

struct CMethodProxy {
    body: CMethodBody,
}

impl CMethodProxy {
    pub fn new(body: CMethodBody) -> Self {
        Self { body }
    }
}

#[async_trait::async_trait(?Send)]
impl EmulatedFunction<(u32, u32, u32), ArmCoreError, Backend, u32> for CMethodProxy {
    async fn call(&self, core: &mut ArmCore, backend: &mut Backend) -> Result<u32, ArmCoreError> {
        let a0 = u32::get(core, 0);
        let a1 = u32::get(core, 1);
        let a2 = u32::get(core, 2);

        let mut context = KtfCContext::new(core, backend);

        // Hack to put lifetime on context.
        let context: &mut KtfCContext<'static> = unsafe { core::mem::transmute(&mut context) };

        self.body.call(context, &[a0, a1, a2]).await
    }
}
