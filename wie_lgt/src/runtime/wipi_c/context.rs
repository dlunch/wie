use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};

use wie_backend::{AsyncCallable, System};
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, EmulatedFunctionParam, ResultWriter};
use wie_util::{ByteRead, ByteWrite, Result, read_generic, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};

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
        let address = Allocator::alloc(&mut self.core, size + size_of::<WIPICWord>() as WIPICWord)?;
        write_generic(&mut self.core, address, size)?;

        Ok(WIPICMemoryId(address + size_of::<WIPICWord>() as WIPICWord))
    }

    fn free(&mut self, memory: WIPICMemoryId) -> Result<()> {
        let base_address = memory.0 - size_of::<WIPICWord>() as WIPICWord;

        let size: WIPICWord = read_generic(&self.core, base_address)?;
        Allocator::free(&mut self.core, base_address, size + size_of::<WIPICWord>() as WIPICWord)
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> Result<WIPICWord> {
        Ok(memory.0)
    }

    fn register_function(&mut self, body: WIPICMethodBody) -> Result<WIPICWord> {
        struct WIPICMethodResult {
            result: WIPICResult,
        }

        impl ResultWriter<WIPICMethodResult> for WIPICMethodResult {
            fn write(self, core: &mut ArmCore, next_pc: u32) -> Result<()> {
                core.write_return_value(&self.result.results)?;
                core.set_next_pc(next_pc)?;

                Ok(())
            }
        }

        struct CMethodProxy {
            context: LgtWIPICContext,
            body: WIPICMethodBody,
        }

        #[async_trait::async_trait]
        impl EmulatedFunction<(), WIPICMethodResult, ()> for CMethodProxy {
            async fn call(&self, core: &mut ArmCore, _: &mut ()) -> Result<WIPICMethodResult> {
                let a0 = u32::get(core, 0);
                let a1 = u32::get(core, 1);
                let a2 = u32::get(core, 2);
                let a3 = u32::get(core, 3);
                let a4 = u32::get(core, 4);
                let a5 = u32::get(core, 5);
                let a6 = u32::get(core, 6);
                let a7 = u32::get(core, 7);
                let a8 = u32::get(core, 8); // TODO create arg proxy

                let result = self
                    .body
                    .call(&mut self.context.clone(), vec![a0, a1, a2, a3, a4, a5, a6, a7, a8].into_boxed_slice())
                    .await?;

                Ok(WIPICMethodResult { result })
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

        impl AsyncCallable<Result<()>> for SpawnProxy {
            async fn call(mut self) -> Result<()> {
                self.context.jvm.attach_thread().unwrap();
                self.callback.call(&mut self.context, Box::new([])).await?;
                self.context.jvm.detach_thread().unwrap();

                Ok(())
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
