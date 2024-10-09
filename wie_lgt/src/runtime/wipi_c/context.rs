use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{
    runtime::{JavaIoInputStream, JavaLangClassLoader},
    Jvm,
};

use wie_backend::{AsyncCallable, System};
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam};
use wie_util::{ByteRead, ByteWrite, Result};
use wie_wipi_c::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICWord};

// mostly same as ktf's one, can we merge those?
#[derive(Clone)]
pub struct LgtWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm,
}

impl LgtWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
    }
}

#[async_trait::async_trait]
impl WIPICContext for LgtWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        Allocator::alloc(&mut self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICMemoryId> {
        Allocator::alloc(&mut self.core, size).map(WIPICMemoryId)
    }

    fn free(&mut self, _memory: WIPICMemoryId) -> Result<()> {
        todo!()
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> Result<WIPICWord> {
        Ok(memory.0)
    }

    fn register_function(&mut self, body: WIPICMethodBody) -> Result<WIPICWord> {
        struct CMethodProxy {
            context: LgtWIPICContext,
            body: WIPICMethodBody,
        }

        #[async_trait::async_trait]
        impl EmulatedFunction<(), u32, ()> for CMethodProxy {
            async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<u32> {
                let a0 = u32::get(core, 0);
                let a1 = u32::get(core, 1);
                let a2 = u32::get(core, 2);
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);
                let a7 = u32::get(core, 7);
                let a8 = u32::get(core, 8); // TODO create arg proxy

                self.body
                    .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
                    .await
            }
        }

        let proxy = CMethodProxy { context: self.clone(), body };

        self.core.register_function(proxy, &())
    }

    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord> {
        self.core.run_function(address, args).await
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()> {
        struct SpawnProxy {
            context: LgtWIPICContext,
            callback: WIPICMethodBody,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<WIPICWord>> for SpawnProxy {
            async fn call(mut self) -> Result<WIPICWord> {
                // self.context.jvm.attach_thread().await.unwrap();
                /* let result = */
                self.callback.call(&mut self.context, Box::new([])).await
                // self.context.jvm.detach_thread().await.unwrap();
            }
        }

        self.system.spawn(SpawnProxy {
            context: self.clone(),
            callback,
        });

        Ok(())
    }

    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>> {
        let class_loader = self.jvm.current_class_loader().await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if stream.is_none() {
            return Ok(None);
        }

        let available: i32 = self.jvm.invoke_virtual(&stream.unwrap(), "available", "()I", ()).await.unwrap();

        Ok(Some(available as _))
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        let class_loader = self.jvm.current_class_loader().await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name)
            .await
            .unwrap()
            .unwrap();

        Ok(JavaIoInputStream::read_until_end(&self.jvm, &stream).await.unwrap())
    }
}

impl ByteRead for LgtWIPICContext {
    fn read_bytes(&self, address: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, result)
    }
}

impl ByteWrite for LgtWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}
