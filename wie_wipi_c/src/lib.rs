#![no_std]
extern crate alloc;

pub mod api;
mod context;
mod error;
mod method;

pub use self::{context::WIPICContext, error::WIPICError};

use alloc::boxed::Box;

use bytemuck::{Pod, Zeroable};

use crate::method::MethodBody;

pub type WIPICResult<T> = core::result::Result<T, WIPICError>;
pub type WIPICWord = u32; // wipi c is 32bit target
pub type WIPICMethodBody = Box<dyn MethodBody<WIPICError>>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct WIPICMemoryId(pub WIPICWord);
