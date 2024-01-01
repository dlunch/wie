use alloc::{boxed::Box, vec, vec::Vec};

use wie_backend::{
    task::{self, SleepFuture},
    AsyncCallable, SystemHandle,
};
use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::{Allocator, ArmCore, ArmEngineError, EmulatedFunction, EmulatedFunctionParam};
use wie_impl_wipi_c::{WIPICContext, WIPICError, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};

pub struct KtfWIPICContext<'a> {
    core: &'a mut ArmCore,
    system: &'a mut SystemHandle,
}

impl<'a> KtfWIPICContext<'a> {
    pub fn new(core: &'a mut ArmCore, system: &'a mut SystemHandle) -> Self {
        Self { core, system }
    }
}

#[async_trait::async_trait(?Send)]
impl WIPICContext for KtfWIPICContext<'_> {
    fn alloc_raw(&mut self, size: WIPICWord) -> WIPICResult<WIPICWord> {
        Allocator::alloc(self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> WIPICResult<WIPICMemoryId> {
        let ptr = Allocator::alloc(self.core, size + 12)?; // all allocation has indirect pointer
        write_generic(self.core, ptr, ptr + 4)?;

        Ok(WIPICMemoryId(ptr))
    }

    fn free(&mut self, memory: WIPICMemoryId) -> WIPICResult<()> {
        Allocator::free(self.core, memory.0)
    }

    fn free_raw(&mut self, address: WIPICWord) -> WIPICResult<()> {
        Allocator::free(self.core, address)
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> WIPICResult<WIPICWord> {
        let base: WIPICWord = read_generic(self.core, memory.0)?;

        Ok(base + 8) // all data has offset of 8 bytes
    }

    fn register_function(&mut self, body: WIPICMethodBody) -> WIPICResult<WIPICWord> {
        struct CMethodProxy {
            body: WIPICMethodBody,
        }

        impl CMethodProxy {
            pub fn new(body: WIPICMethodBody) -> Self {
                Self { body }
            }
        }

        #[async_trait::async_trait(?Send)]
        impl EmulatedFunction<(), ArmEngineError, u32> for CMethodProxy {
            async fn call(&self, core: &mut ArmCore, system: &mut SystemHandle) -> Result<u32, ArmEngineError> {
                let a0 = u32::get(core, 0);
                let a1 = u32::get(core, 1);
                let a2 = u32::get(core, 2);
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);
                let a7 = u32::get(core, 7);
                let a8 = u32::get(core, 8); // TODO create arg proxy

                let mut context = KtfWIPICContext::new(core, system);

                self.body
                    .call(&mut context, vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
                    .await
            }
        }

        let proxy = CMethodProxy::new(body);

        self.core.register_function(proxy)
    }

    fn system(&mut self) -> &mut SystemHandle {
        self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> WIPICResult<WIPICWord> {
        self.core.run_function(address, args).await
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> WIPICResult<()> {
        struct SpawnProxy {
            core: ArmCore,
            system: SystemHandle,
            callback: WIPICMethodBody,
        }

        #[async_trait::async_trait(?Send)]
        impl AsyncCallable<WIPICWord, WIPICError> for SpawnProxy {
            #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
            async fn call(mut self) -> Result<WIPICWord, WIPICError> {
                let mut context = KtfWIPICContext::new(&mut self.core, &mut self.system);

                self.callback.call(&mut context, Box::new([])).await
            }
        }

        let system = self.system.clone();

        self.core.spawn(SpawnProxy {
            core: self.core.clone(),
            system,
            callback,
        });

        Ok(())
    }

    fn sleep(&mut self, duration: u64) -> SleepFuture {
        let until = self.system.platform().now() + duration;

        task::sleep(until)
    }
}

impl ByteRead for KtfWIPICContext<'_> {
    fn read_bytes(&self, address: WIPICWord, size: WIPICWord) -> anyhow::Result<Vec<u8>> {
        self.core.read_bytes(address, size)
    }
}

impl ByteWrite for KtfWIPICContext<'_> {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> anyhow::Result<()> {
        self.core.write_bytes(address, data)
    }
}
