use alloc::{boxed::Box, string::String};

use bytemuck::{Pod, Zeroable};

use wie_backend::{task::SleepFuture, Backend};
use wie_base::util::{read_null_terminated_string, ByteRead, ByteWrite};

use crate::method::{MethodBody, TypeConverter};

pub type WIPICError = anyhow::Error;
pub type WIPICResult<T> = anyhow::Result<T>;
pub type WIPICWord = u32; // wipi c is 32bit target
pub type WIPICMethodBody = Box<dyn MethodBody<WIPICError>>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct WIPICMemoryId(pub WIPICWord);

#[async_trait::async_trait(?Send)]
pub trait WIPICContext: ByteRead + ByteWrite {
    fn alloc_raw(&mut self, size: WIPICWord) -> WIPICResult<WIPICWord>;
    fn alloc(&mut self, size: WIPICWord) -> WIPICResult<WIPICMemoryId>;
    fn free(&mut self, memory: WIPICMemoryId) -> WIPICResult<()>;
    fn free_raw(&mut self, address: WIPICWord) -> WIPICResult<()>;
    fn data_ptr(&self, memory: WIPICMemoryId) -> WIPICResult<WIPICWord>;
    fn register_function(&mut self, method: WIPICMethodBody) -> WIPICResult<WIPICWord>;
    async fn call_method(&mut self, address: WIPICWord, args: &[WIPICWord]) -> WIPICResult<WIPICWord>;
    fn backend(&mut self) -> &mut Backend;
    fn spawn(&mut self, callback: WIPICMethodBody) -> WIPICResult<()>;
    fn sleep(&mut self, duration: u64) -> SleepFuture;
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
