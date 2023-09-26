extern crate alloc;

mod backend;
mod executor;
pub mod task;

pub use self::{
    backend::{canvas, window::Window, Backend},
    executor::{AsyncCallable, Executor},
};
