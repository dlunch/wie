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
    async fn get_resource_size(&self, name: &str) -> Result<usize>;
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
