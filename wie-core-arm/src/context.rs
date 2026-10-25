use alloc::vec::Vec;
use core::mem::{offset_of, size_of};

use bytemuck::{Pod, Zeroable};

use wie_util::{Result, WieError, read_generic, write_generic};

use crate::{Allocator, ArmCore, core::ACTIVE_THREAD, thread::ThreadControl};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ArmCoreContext {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub sb: u32,
    pub sl: u32,
    pub fp: u32,
    pub ip: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
    pub cpsr: u32,
}

pub struct ArmRootSnapshot {
    pub words: Vec<u32>,
    pub runtime_contexts: Vec<u32>,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SavedContextRecord {
    pub context: ArmCoreContext,
    pub next: u32,
}

pub struct SavedArmContext {
    core: ArmCore,
    control_address: u32,
    address: u32,
}

impl ArmCore {
    pub fn save_resumable_context(&mut self) -> Result<SavedArmContext> {
        let control_address: u32 = read_generic(self, ACTIVE_THREAD)?;
        if control_address == 0 {
            return Err(WieError::FatalError("Resumable ARM execution requires an entered thread".into()));
        }
        let head = control_address + offset_of!(ThreadControl, saved_contexts) as u32;
        let record = SavedContextRecord {
            context: self.save_context(),
            next: read_generic(self, head)?,
        };
        let address = Allocator::alloc(self, size_of::<SavedContextRecord>() as u32)?;
        if let Err(error) = write_generic(self, address, record).and_then(|()| write_generic(self, head, address)) {
            Allocator::free(self, address, size_of::<SavedContextRecord>() as u32)?;
            return Err(error);
        }
        Ok(SavedArmContext {
            core: self.clone(),
            control_address,
            address,
        })
    }
}

impl SavedArmContext {
    pub fn read(&self) -> Result<ArmCoreContext> {
        let record: SavedContextRecord = read_generic(&self.core, self.address)?;
        Ok(record.context)
    }

    fn unlink(&mut self) -> Result<()> {
        let mut link = self.control_address + offset_of!(ThreadControl, saved_contexts) as u32;
        let mut address: u32 = read_generic(&self.core, link)?;
        while address != self.address {
            link = address + offset_of!(SavedContextRecord, next) as u32;
            address = read_generic(&self.core, link)?;
        }
        let record: SavedContextRecord = read_generic(&self.core, self.address)?;
        write_generic(&mut self.core, link, record.next)?;
        Allocator::free(&mut self.core, self.address, size_of::<SavedContextRecord>() as u32)
    }
}

impl Drop for SavedArmContext {
    fn drop(&mut self) {
        if let Err(error) = self.unlink() {
            tracing::error!("Failed to release saved ARM context: {error}");
        }
    }
}
