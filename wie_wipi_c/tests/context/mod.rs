use wie_backend::System;
use wie_util::{ByteRead, ByteWrite};
use wie_wipi_c::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord};

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

#[async_trait::async_trait(?Send)]
impl WIPICContext for TestContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> WIPICResult<WIPICWord> {
        let address = self.last_alloc;
        self.last_alloc += size as usize;

        Ok(address as WIPICWord)
    }

    fn alloc(&mut self, size: WIPICWord) -> WIPICResult<WIPICMemoryId> {
        Ok(WIPICMemoryId(Self::alloc_raw(self, size)?))
    }

    fn free(&mut self, _memory: WIPICMemoryId) -> WIPICResult<()> {
        Ok(())
    }

    fn free_raw(&mut self, _address: WIPICWord) -> WIPICResult<()> {
        Ok(())
    }

    fn data_ptr(&self, memory: WIPICMemoryId) -> WIPICResult<WIPICWord> {
        Ok(memory.0)
    }

    fn register_function(&mut self, _method: WIPICMethodBody) -> WIPICResult<WIPICWord> {
        todo!()
    }

    async fn call_function(&mut self, _address: WIPICWord, _args: &[WIPICWord]) -> WIPICResult<WIPICWord> {
        todo!()
    }

    fn system(&mut self) -> &mut System {
        todo!()
    }

    fn spawn(&mut self, _callback: WIPICMethodBody) -> WIPICResult<()> {
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
    fn read_bytes(&self, address: u32, size: u32) -> wie_util::Result<Vec<u8>> {
        Ok(self.memory[address as usize..(address + size) as usize].to_vec())
    }
}
