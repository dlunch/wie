mod graphics;
mod kernel;

pub use graphics::get_graphics_method_table;
pub use kernel::get_kernel_method_table;

use super::method::TypeConverter;
pub use super::method::{MethodBody, MethodImpl};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;

pub type CBridgeMethod = Box<dyn Fn(&mut CContext, Vec<u32>) -> CResult<u32>>;
pub type CMethodBody = Box<dyn MethodBody<CError, CContext>>;

pub type CContext = dyn CBridge;

pub trait CBridge {
    fn alloc(&mut self, size: u32) -> CResult<u32>;
    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()>;
    fn register_function(&mut self, method: CBridgeMethod) -> CResult<u32>;
}

fn into_body<M, F, R, P>(method: M) -> CMethodBody
where
    M: MethodImpl<F, R, CError, CContext, P>,
{
    method.into_body()
}

impl TypeConverter<u32, CContext> for u32 {
    fn to_rust(_: &mut CContext, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut CContext, rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<(), CContext> for () {
    fn to_rust(_: &mut CContext, _: u32) {}

    fn from_rust(_: &mut CContext, _: ()) -> u32 {
        0
    }
}
