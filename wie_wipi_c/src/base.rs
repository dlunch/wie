use alloc::{boxed::Box, string::String};

use bytemuck::{Pod, Zeroable};

use wie_backend::{task::SleepFuture, Backend};
use wie_base::util::{read_null_terminated_string, ByteRead, ByteWrite};

use crate::method::{MethodBody, TypeConverter};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;
pub type CWord = u32; // wipi c is 32bit target
pub type CMethodBody = Box<dyn MethodBody<CError>>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CMemoryId(pub CWord);

#[async_trait::async_trait(?Send)]
pub trait CContext: ByteRead + ByteWrite {
    fn alloc_raw(&mut self, size: CWord) -> CResult<CWord>;
    fn alloc(&mut self, size: CWord) -> CResult<CMemoryId>;
    fn free(&mut self, memory: CMemoryId) -> CResult<()>;
    fn data_ptr(&self, memory: CMemoryId) -> CResult<CWord>;
    fn register_function(&mut self, method: CMethodBody) -> CResult<CWord>;
    async fn call_method(&mut self, address: CWord, args: &[CWord]) -> CResult<CWord>;
    fn backend(&mut self) -> &mut Backend;
    fn spawn(&mut self, callback: CMethodBody) -> CResult<()>;
    fn sleep(&mut self, duration: u64) -> SleepFuture;
}

impl TypeConverter<CWord> for CWord {
    fn to_rust(_: &mut dyn CContext, raw: CWord) -> CWord {
        raw
    }

    fn from_rust(_: &mut dyn CContext, rust: CWord) -> CWord {
        rust
    }
}

impl TypeConverter<CMemoryId> for CMemoryId {
    fn to_rust(_: &mut dyn CContext, raw: CWord) -> CMemoryId {
        CMemoryId(raw)
    }

    fn from_rust(_: &mut dyn CContext, rust: CMemoryId) -> CWord {
        rust.0
    }
}

impl TypeConverter<i32> for i32 {
    fn to_rust(_: &mut dyn CContext, raw: CWord) -> i32 {
        raw as _
    }

    fn from_rust(_: &mut dyn CContext, rust: i32) -> CWord {
        rust as _
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: &mut dyn CContext, _: CWord) {}

    fn from_rust(_: &mut dyn CContext, _: ()) -> CWord {
        0
    }
}

impl TypeConverter<String> for String {
    fn to_rust(context: &mut dyn CContext, raw: CWord) -> String {
        read_null_terminated_string(context, raw).unwrap()
    }

    fn from_rust(_: &mut dyn CContext, _: String) -> CWord {
        unimplemented!()
    }
}
