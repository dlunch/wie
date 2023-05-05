mod graphics;
mod kernel;
mod method;

pub use graphics::get_graphics_method_table;
pub use kernel::get_kernel_method_table;

pub use self::method::{CMethodBody, CMethodImpl};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;

pub type CBridgeMethod = Box<dyn Fn(&mut dyn CBridge) -> CResult<u32>>;
pub trait CBridge {
    fn alloc(&mut self, size: u32) -> CResult<u32>;
    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()>;
    fn register_function(&mut self, method: CBridgeMethod) -> CResult<u32>;
}

fn into_body<M, F, R, P>(method: M) -> Box<dyn CMethodBody<CError>>
where
    M: CMethodImpl<F, CError, R, P>,
{
    method.into_body()
}
