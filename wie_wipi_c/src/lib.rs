#![no_std]
extern crate alloc;

pub mod api;
mod context;
mod method;

pub use self::context::WIPICContext;

use alloc::boxed::Box;

use bytemuck::{Pod, Zeroable};

use wie_util::WieError;

use crate::method::MethodBody;

pub type WIPICWord = u32; // wipi c is 32bit target
pub type WIPICMethodBody = Box<dyn MethodBody<WieError>>;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct WIPICMemoryId(pub WIPICWord);
