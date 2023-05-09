mod graphics;
mod kernel;

pub use graphics::get_graphics_method_table;
pub use kernel::get_kernel_method_table;

use crate::{
    backend::Backend,
    util::{read_null_terminated_string, ByteRead, ByteWrite},
};

use super::method::TypeConverter;
pub use super::method::{MethodBody, MethodImpl};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;

pub type CContextMethod = Box<dyn Fn(&mut CContext, Vec<u32>) -> CResult<u32>>;
pub type CMethodBody = Box<dyn MethodBody<CError, CContext>>;

pub type CContext = dyn CContextBase;

pub trait CContextBase: ByteRead + ByteWrite {
    fn alloc(&mut self, size: u32) -> CResult<u32>;
    fn register_function(&mut self, method: CContextMethod) -> CResult<u32>;
    fn backend(&mut self) -> &mut Backend;
}

impl TypeConverter<u32, CContext> for u32 {
    fn to_rust(_: &mut CContext, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut CContext, rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<i32, CContext> for i32 {
    fn to_rust(_: &mut CContext, raw: u32) -> i32 {
        raw as _
    }

    fn from_rust(_: &mut CContext, rust: i32) -> u32 {
        rust as _
    }
}

impl TypeConverter<(), CContext> for () {
    fn to_rust(_: &mut CContext, _: u32) {}

    fn from_rust(_: &mut CContext, _: ()) -> u32 {
        0
    }
}

impl TypeConverter<String, CContext> for String {
    fn to_rust(context: &mut CContext, raw: u32) -> String {
        read_null_terminated_string(context, raw).unwrap()
    }

    fn from_rust(_: &mut CContext, _: String) -> u32 {
        unimplemented!()
    }
}
