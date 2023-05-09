#![allow(dead_code)]

use crate::{backend::Task, core::arm::ArmCore};

pub struct KtfTaskContext {
    regs: [u32; 12],
}

impl KtfTaskContext {
    pub fn from_pc(pc: u32) -> Self {
        Self {
            regs: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, pc],
        }
    }
}

pub struct KtfTask {
    context: KtfTaskContext,
    core: ArmCore,
}

impl KtfTask {
    pub fn new(core: ArmCore, context: KtfTaskContext) -> Self {
        Self { context, core }
    }

    fn restore_context(&mut self) {
        unimplemented!();
    }

    fn save_context(&mut self) {
        unimplemented!()
    }
}

impl Task for KtfTask {
    fn run_some(&mut self) {
        self.restore_context();

        log::debug!("KtfTask::run_some");

        self.save_context();
    }
}
