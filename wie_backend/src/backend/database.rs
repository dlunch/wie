#[cfg(not(target_arch = "wasm32"))]
mod std;
#[cfg(not(target_arch = "wasm32"))]
pub use std::DatabaseRepository;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::DatabaseRepository;

pub type RecordId = u32;
