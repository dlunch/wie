use alloc::{boxed::Box, vec, vec::Vec};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{Instant, System};
use wie_util::{ByteRead, ByteWrite, Result};

use crate::{
    WIPICMethodBody,
    method::{ParamConverter, ResultConverter},
};

#[async_trait::async_trait]
pub trait WIPICContext: ByteRead + ByteWrite + Send {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord>;
    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr>;
    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()>;
    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()>;
    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord>;
    fn register_function(&mut self, method: WIPICMethodBody) -> Result<WIPICWord>;
    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord>;
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()>;
    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>>;
    async fn read_resource(&self, name: &str) -> Result<Vec<u8>>;
    fn set_timer(&mut self, due: Instant, callback: WIPICMethodBody);
}

pub struct WIPICResult {
    pub results: Vec<WIPICWord>,
}

impl ParamConverter<WIPICWord> for WIPICWord {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICWord {
        raw
    }
}

impl ParamConverter<WIPICIndirectPtr> for WIPICIndirectPtr {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICIndirectPtr {
        WIPICIndirectPtr(raw)
    }
}

impl ParamConverter<i32> for i32 {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> i32 {
        raw as _
    }
}

impl ResultConverter<u64> for u64 {
    fn convert(_: &mut dyn WIPICContext, result: u64) -> WIPICResult {
        WIPICResult {
            results: vec![result as u32, (result << 32) as u32],
        }
    }
}

impl ResultConverter<WIPICWord> for WIPICWord {
    fn convert(_: &mut dyn WIPICContext, result: WIPICWord) -> WIPICResult {
        WIPICResult { results: vec![result] }
    }
}

impl ResultConverter<WIPICIndirectPtr> for WIPICIndirectPtr {
    fn convert(_: &mut dyn WIPICContext, result: WIPICIndirectPtr) -> WIPICResult {
        WIPICResult { results: vec![result.0] }
    }
}

impl ResultConverter<i32> for i32 {
    fn convert(_: &mut dyn WIPICContext, result: i32) -> WIPICResult {
        WIPICResult { results: vec![result as _] }
    }
}

impl ResultConverter<()> for () {
    fn convert(_: &mut dyn WIPICContext, _: ()) -> WIPICResult {
        WIPICResult { results: Vec::new() }
    }
}

#[cfg(test)]
pub mod test {
    use alloc::{boxed::Box, vec::Vec};

    use wie_backend::{Instant, System};
    use wie_util::{ByteRead, ByteWrite, Result};

    use crate::{WIPICContext, WIPICIndirectPtr, WIPICMethodBody, WIPICWord};

    pub struct TestContext {
        memory: [u8; 0x10000],
        last_alloc: usize,
    }

    impl TestContext {
        #[allow(clippy::new_without_default)]
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

        fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
            Ok(WIPICIndirectPtr(Self::alloc_raw(self, size)?))
        }

        fn free(&mut self, _memory: WIPICIndirectPtr) -> Result<()> {
            Ok(())
        }

        fn free_raw(&mut self, _address: WIPICWord, _size: WIPICWord) -> Result<()> {
            Ok(())
        }

        fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
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

        async fn get_resource_size(&self, _name: &str) -> Result<Option<usize>> {
            todo!()
        }

        async fn read_resource(&self, _name: &str) -> Result<Vec<u8>> {
            todo!()
        }

        fn set_timer(&mut self, _due: Instant, _callback: WIPICMethodBody) {
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
}
