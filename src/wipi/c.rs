mod graphics;
mod method;

pub use graphics::get_graphics_method_table;

use self::method::{CMethodBody, CMethodImpl};

pub type CError = anyhow::Error;
pub type CResult<T> = anyhow::Result<T>;

pub trait Bridge {}

fn into_body<M, F, R, P>(method: M) -> Box<dyn CMethodBody<CError>>
where
    M: CMethodImpl<F, CError, R, P>,
{
    method.into_body()
}
