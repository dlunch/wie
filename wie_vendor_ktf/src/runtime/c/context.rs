use alloc::{boxed::Box, vec::Vec};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, Backend,
};
use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, ArmCoreError, EmulatedFunction, EmulatedFunctionParam};
use wie_wipi_c::{CContext, CError, CMemoryId, CMethodBody, CResult};

pub struct KtfCContext<'a> {
    core: &'a mut ArmCore,
    backend: &'a mut Backend,
}

impl<'a> KtfCContext<'a> {
    pub fn new(core: &'a mut ArmCore, backend: &'a mut Backend) -> Self {
        Self { core, backend }
    }
}

#[async_trait::async_trait(?Send)]
impl CContext for KtfCContext<'_> {
    fn alloc_raw(&mut self, size: u32) -> CResult<u32> {
        Allocator::alloc(self.core, size)
    }

    fn alloc(&mut self, size: u32) -> CResult<CMemoryId> {
        let ptr = Allocator::alloc(self.core, size + 12)?; // all allocation has indirect pointer
        write_generic(self.core, ptr, ptr + 4)?;

        Ok(CMemoryId(ptr))
    }

    fn free(&mut self, memory: CMemoryId) -> CResult<()> {
        Allocator::free(self.core, memory.0)
    }

    fn data_ptr(&self, memory: CMemoryId) -> CResult<u32> {
        let base: u32 = read_generic(self.core, memory.0)?;

        Ok(base + 8) // all data has offset of 8 bytes
    }

    fn register_function(&mut self, body: CMethodBody) -> CResult<u32> {
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
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);
                let a7 = u32::get(core, 7);
                let a8 = u32::get(core, 8); // TODO create arg proxy

                let mut context = KtfCContext::new(core, backend);

                self.body.call(&mut context, &[a0, a1, a2, a3, a4, a5, a6, a7, a8]).await
            }
        }

        let proxy = CMethodProxy::new(body);

        self.core.register_function(proxy, self.backend)
    }

    fn backend(&mut self) -> &mut Backend {
        self.backend
    }

    async fn call_method(&mut self, address: u32, args: &[u32]) -> CResult<u32> {
        self.core.run_function(address, args).await
    }

    fn spawn(&mut self, callback: CMethodBody) -> CResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            backend: Backend,
            callback: CMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<u32, CError> for SpawnProxy {
            #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
            async fn call(mut self) -> Result<u32, CError> {
                let mut context = KtfCContext::new(&mut self.core, &mut self.backend);

                self.callback.call(&mut context, &[]).await
            }
        }

        let backend = self.backend.clone();

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            backend,
            callback,
        });

        Ok(())
    }

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.backend.time().now() + duration;

        task::sleep(until)
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
