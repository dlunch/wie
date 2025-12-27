#![no_std]
extern crate alloc;

pub mod api;
mod context;
mod method;

pub use self::context::{WIPICContext, WIPICResult};
pub use self::method::MethodImpl;

use alloc::boxed::Box;

use wie_util::WieError;

use crate::method::MethodBody;

pub type WIPICMethodBody = Box<dyn MethodBody<WieError>>;
