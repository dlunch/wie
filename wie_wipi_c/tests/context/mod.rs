use wie_backend::System;
use wie_util::{ByteRead, ByteWrite, Result};
use wie_wipi_c::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICWord};

pub struct TestContext {
    memory: [u8; 0x10000],
    last_alloc: usize,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000],
            last_alloc: 0,
        }
    }
}

#[async_trait::async_trait]
impl WIPICContext for TestContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        let address = self.last_alloc;
        self.last_alloc += size as usize;

        Ok(address as WIPICWord)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICMemoryId> {
        Ok(WIPICMemoryId(Self::alloc_raw(self, size)?))
    }

    fn free(&mut self, _memory: WIPICMemoryId) -> Result<()> {
        Ok(())
    }

    fn free_raw(&mut self, _address: WIPICWord, _size: WIPICWord) -> Result<()> {
        Ok(())
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> Result<WIPICWord> {
        Ok(memory.0)
    }

    fn register_function(&mut self, _method: WIPICMethodBody) -> Result<WIPICWord> {
        todo!()
    }

    async fn call_function(&mut self, _address: WIPICWord, _args: &[WIPICWord]) -> Result<WIPICWord> {
        todo!()
    }

    fn system(&mut self) -> &mut System {
        todo!()
    }

    fn spawn(&mut self, _callback: WIPICMethodBody) -> Result<()> {
        todo!()
    }

    async fn get_resource_size(&self, _name: &str) -> Result<usize> {
        todo!()
    }

    async fn read_resource(&self, _name: &str) -> Result<Vec<u8>> {
        todo!()
    }
}

impl ByteWrite for TestContext {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        self.memory[address as usize..(address + data.len() as u32) as usize].copy_from_slice(data);

        Ok(())
    }
}

impl ByteRead for TestContext {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> wie_util::Result<usize> {
        result.copy_from_slice(&self.memory[address as usize..(address as usize + result.len())]);

        Ok(result.len())
    }
}
