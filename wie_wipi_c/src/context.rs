use alloc::{boxed::Box, string::String, vec::Vec};

use wie_backend::System;
use wie_util::{read_null_terminated_string, ByteRead, ByteWrite, Result};

use crate::{method::TypeConverter, WIPICMemoryId, WIPICMethodBody, WIPICWord};

#[async_trait::async_trait]
pub trait WIPICContext: ByteRead + ByteWrite + Send {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord>;
    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICMemoryId>;
    fn free(&mut self, memory: WIPICMemoryId) -> Result<()>;
    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()>;
    fn data_ptr(&self, memory: WIPICMemoryId) -> Result<WIPICWord>;
    fn register_function(&mut self, method: WIPICMethodBody) -> Result<WIPICWord>;
    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord>;
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()>;
    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>>;
    async fn read_resource(&self, name: &str) -> Result<Vec<u8>>;
}

impl TypeConverter<WIPICWord> for WIPICWord {
    fn to_rust(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICWord {
        raw
    }

    fn from_rust(_: &mut dyn WIPICContext, rust: WIPICWord) -> WIPICWord {
        rust
    }
}

impl TypeConverter<WIPICMemoryId> for WIPICMemoryId {
    fn to_rust(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICMemoryId {
        WIPICMemoryId(raw)
    }

    fn from_rust(_: &mut dyn WIPICContext, rust: WIPICMemoryId) -> WIPICWord {
        rust.0
    }
}

impl TypeConverter<i32> for i32 {
    fn to_rust(_: &mut dyn WIPICContext, raw: WIPICWord) -> i32 {
        raw as _
    }

    fn from_rust(_: &mut dyn WIPICContext, rust: i32) -> WIPICWord {
        rust as _
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: &mut dyn WIPICContext, _: WIPICWord) {}

    fn from_rust(_: &mut dyn WIPICContext, _: ()) -> WIPICWord {
        0
    }
}

impl TypeConverter<String> for String {
    fn to_rust(context: &mut dyn WIPICContext, raw: WIPICWord) -> String {
        read_null_terminated_string(context, raw).unwrap()
    }

    fn from_rust(_: &mut dyn WIPICContext, _: String) -> WIPICWord {
        unimplemented!()
    }
}

#[cfg(test)]
pub mod test {
    use alloc::{boxed::Box, vec::Vec};

    use wie_backend::System;
    use wie_util::{ByteRead, ByteWrite, Result};

    use crate::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICWord};

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
}
