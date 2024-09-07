use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{
    runtime::{JavaIoInputStream, JavaLangClassLoader},
    Jvm,
};

use wie_backend::{AsyncCallable, System};
use wie_core_arm::{Allocator, ArmCore, ArmCoreError, EmulatedFunction, EmulatedFunctionParam};
use wie_util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_wipi_c::{WIPICContext, WIPICError, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};

#[derive(Clone)]
pub struct KtfWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm, // We need jvm to access resource in jvm. TODO is there better way to do this?
}

impl KtfWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
    }
}

#[async_trait::async_trait]
impl WIPICContext for KtfWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> WIPICResult<WIPICWord> {
        Ok(Allocator::alloc(&mut self.core, size).unwrap())
    }

    fn alloc(&mut self, size: WIPICWord) -> WIPICResult<WIPICMemoryId> {
        let ptr = Allocator::alloc(&mut self.core, size + 12).unwrap(); // all allocation has indirect pointer
        write_generic(&mut self.core, ptr, ptr + 4)?;

        Ok(WIPICMemoryId(ptr))
    }

    fn free(&mut self, memory: WIPICMemoryId) -> WIPICResult<()> {
        Allocator::free(&mut self.core, memory.0).unwrap();

        Ok(())
    }

    fn free_raw(&mut self, address: WIPICWord) -> WIPICResult<()> {
        Allocator::free(&mut self.core, address).unwrap();

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> WIPICResult<WIPICWord> {
        let base: WIPICWord = read_generic(&self.core, memory.0)?;

        Ok(base + 8) // all data has offset of 8 bytes
    }

    fn register_function(&mut self, body: WIPICMethodBody) -> WIPICResult<WIPICWord> {
        struct CMethodProxy {
            context: KtfWIPICContext,
            body: WIPICMethodBody,
        }

        #[async_trait::async_trait]
        impl EmulatedFunction<(), u32, ArmCoreError, ()> for CMethodProxy {
            async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<u32, ArmCoreError> {
                let a0 = u32::get(core, 0);
                let a1 = u32::get(core, 1);
                let a2 = u32::get(core, 2);
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);
                let a7 = u32::get(core, 7);
                let a8 = u32::get(core, 8); // TODO create arg proxy

                Ok(self
                    .body
                    .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
                    .await
                    .unwrap())
            }
        }

        let proxy = CMethodProxy { context: self.clone(), body };

        Ok(self.core.register_function(proxy, &()).unwrap())
    }

    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> WIPICResult<WIPICWord> {
        Ok(self.core.run_function(address, args).await.unwrap())
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> WIPICResult<()> {
        struct SpawnProxy {
            context: KtfWIPICContext,
            callback: WIPICMethodBody,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<WIPICWord, WIPICError>> for SpawnProxy {
            async fn call(mut self) -> Result<WIPICWord, WIPICError> {
                self.context.jvm.attach_thread().await.unwrap();
                let result = self.callback.call(&mut self.context, Box::new([])).await;
                self.context.jvm.detach_thread().await.unwrap();

                result
            }
        }

        self.system.spawn(SpawnProxy {
            context: self.clone(),
            callback,
        });

        Ok(())
    }

    async fn get_resource_size(&self, name: &str) -> WIPICResult<usize> {
        let class_loader = self.jvm.current_class_loader().await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        let available: i32 = self.jvm.invoke_virtual(&stream.unwrap(), "available", "()I", ()).await.unwrap();

        Ok(available as _)
    }

    async fn read_resource(&self, name: &str) -> WIPICResult<Vec<u8>> {
        let class_loader = self.jvm.current_class_loader().await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        Ok(JavaIoInputStream::read_until_end(&self.jvm, &stream.unwrap()).await.unwrap())
    }
}

impl ByteRead for KtfWIPICContext {
    fn read_bytes(&self, address: WIPICWord, size: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, size, result)
    }
}

impl ByteWrite for KtfWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}
