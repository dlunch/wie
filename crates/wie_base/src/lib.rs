#![no_std]

extern crate alloc;

pub mod util;

use alloc::{boxed::Box, string::String};
use core::any::Any;

use futures::future::LocalBoxFuture;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub trait Core: AsAny {
    fn new_context(&mut self) -> Box<dyn CoreContext>;
    fn free_context(&mut self, context: Box<dyn CoreContext>);
    fn restore_context(&mut self, context: &dyn CoreContext);
    fn save_context(&self) -> Box<dyn CoreContext>;
    fn dump_reg_stack(&self) -> String;
}

pub trait Module: AsAny {
    fn core_mut(&mut self) -> &mut dyn Core;
    fn start(&mut self) -> LocalBoxFuture<'static, anyhow::Result<()>>;
}

pub trait CoreContext: AsAny {}
