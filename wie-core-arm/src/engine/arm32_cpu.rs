use alloc::{boxed::Box, format, vec::Vec};
use core::{cell::RefCell, ffi::CStr};

use arm32_cpu::{Cpu, Memory, Mode, reg};
use web_time::Instant;

use wie_arm_jit::{AccessResult, CodePageStamp, CompiledExit, ExecutionAccess, RegionKey, RunFrame};
use wie_backend::ProfileSample;
use wie_util::{Result, WieError};

use crate::{
    engine::{ArmEngine, ArmRegister, EngineRunResult, EngineStopReason, MemoryPermission},
    jit::Jit,
};

use super::sampler::Sampler;

pub struct Arm32CpuEngine {
    cpu: Cpu,
    mem: EmulatedMemory,
    sampler: Sampler,
    entry_pc: u32,
    jit: Option<Jit>,
    closed: bool,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "differential.rs"]
mod differential;

#[cfg(test)]
#[path = "cache_tests.rs"]
mod cache_tests;

enum InstructionCacheInvalidation {
    All,
    Address(u32),
}

impl Arm32CpuEngine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: EmulatedMemory::new(),
            sampler: Sampler::new(),
            entry_pc: 0,
            jit: None,
            closed: false,
        }
    }

    pub fn with_backend() -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            use core::sync::atomic::{AtomicU32, Ordering};
            static NEXT_SESSION: AtomicU32 = AtomicU32::new(1);
            let session = u64::from(NEXT_SESSION.fetch_add(1, Ordering::Relaxed));
            let executor = wie_arm_wasm::WasmExecutor::new(session).map_err(WieError::FatalError)?;
            let mut engine = Self::new();
            engine.jit = Some(Jit::new(session, Box::new(executor)));
            Ok(engine)
        }
        #[cfg(not(target_arch = "wasm32"))]
        Ok(Self::new())
    }

    fn is_svc_exception(&self) -> bool {
        self.cpu.reg_get(Mode::User, reg::PC) == 0x08 && (self.cpu.reg_get(Mode::User, reg::CPSR) & 0x1f) == 0x13
    }

    fn read_svc_result(&mut self) -> Result<EngineStopReason> {
        let lr = self.cpu.reg_get(Mode::Supervisor, reg::LR);
        let spsr = self.cpu.reg_get(Mode::Supervisor, reg::SPSR);

        let svc_address = lr.checked_sub(2).ok_or(WieError::InvalidMemoryAccess(lr))?;
        let mut svc_bytes = [0u8; 2];
        self.mem.read_range(svc_address, 2, &mut svc_bytes)?;
        let instruction = u16::from_le_bytes(svc_bytes);
        if instruction & 0xff00 != 0xdf00 {
            return Err(WieError::FatalError(format!(
                "Invalid Thumb SVC instruction {instruction:#06x} at {svc_address:#x}"
            )));
        }

        let category = instruction as u32 & 0xff;

        Ok(EngineStopReason::Svc { category, lr, spsr })
    }

    fn instruction_cache_invalidation(&self, pc: u32, cpsr: u32) -> Option<InstructionCacheInvalidation> {
        if cpsr & 0x20 != 0 {
            return None;
        }
        let page = self.mem.pages[pc as usize / PAGE_SIZE].as_ref()?;
        let offset = (pc & PAGE_MASK & !3) as usize;
        let instruction = u32::from_le_bytes(core::array::from_fn(|index| page.bytes[offset + index])).rotate_right((pc & 3) * 8);
        // ARM926EJ-S CP15 c7: I-cache all/MVA/set-way and combined I/D invalidation.
        if instruction & 0x0fff_0f10 != 0x0e07_0f10 {
            return None;
        }
        let n = cpsr & (1 << 31) != 0;
        let z = cpsr & (1 << 30) != 0;
        let c = cpsr & (1 << 29) != 0;
        let v = cpsr & (1 << 28) != 0;
        let passed = match instruction >> 28 {
            0 => z,
            1 => !z,
            2 => c,
            3 => !c,
            4 => n,
            5 => !n,
            6 => v,
            7 => !v,
            8 => c && !z,
            9 => !c || z,
            10 => n == v,
            11 => n != v,
            12 => !z && n == v,
            13 => z || n != v,
            14 => true,
            _ => false,
        };
        if !passed {
            return None;
        }
        match (instruction & 15, (instruction >> 5) & 7) {
            (5, 0 | 2) | (7, 0) => Some(InstructionCacheInvalidation::All),
            (5, 1) => Some(InstructionCacheInvalidation::Address(
                self.cpu.reg_get(self.cpu.mode(), ((instruction >> 12) & 15) as u8),
            )),
            _ => None,
        }
    }
}

impl ArmEngine for Arm32CpuEngine {
    fn run(&mut self, end: u32, count: u32, deadline: Option<Instant>) -> Result<EngineRunResult> {
        if self.closed {
            return Err(WieError::FatalError("ARM core is shut down".into()));
        }
        if let Some(jit) = &mut self.jit {
            jit.poll(&self.mem);
        }
        let mut instructions_executed = 0;
        let mut interpret_one = false;
        let mut lookup_entry = true;
        let mut check_deadline = true;
        let stop_reason = loop {
            let pc = self.cpu.reg_get(Mode::User, reg::PC);

            if self.is_svc_exception() {
                break self.read_svc_result()?;
            }

            if pc < 0x1000 {
                return Err(WieError::InvalidMemoryAccess(pc));
            }

            if pc == end {
                break EngineStopReason::End;
            }

            if instructions_executed == count {
                break EngineStopReason::Yield;
            }

            if core::mem::take(&mut check_deadline) && deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break EngineStopReason::Deadline;
            }

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            if lookup_entry
                && !interpret_one
                && cpsr & 0x0100_0000 == 0
                && let Some(jit) = &mut self.jit
            {
                let key = RegionKey {
                    pc,
                    thumb: cpsr & 0x20 != 0,
                    cpu_mode: (cpsr & 0x1f) as u8,
                };
                if let Some(handle) = jit.lookup(key, &self.mem) {
                    let mut frame = RunFrame {
                        regs: core::array::from_fn(|index| self.cpu.reg_get(Mode::User, index as u8)),
                        cpsr,
                        end,
                        budget_remaining: count - instructions_executed,
                        sample_remaining: self.sampler.remaining,
                        entry_pc: self.entry_pc,
                        ..RunFrame::default()
                    };
                    let mut access = MemoryAccess {
                        memory: &mut self.mem,
                        sampler: &mut self.sampler,
                    };
                    let exit = match jit.executor.execute(handle, &mut frame, &mut access) {
                        Ok(exit) => exit,
                        Err(error) => {
                            self.shutdown();
                            return Err(WieError::FatalError(format!("ARM JIT execution failed: {error}")));
                        }
                    };
                    for (index, value) in frame.regs.into_iter().enumerate() {
                        self.cpu.reg_set(Mode::User, index as u8, value);
                    }
                    self.cpu.reg_set(Mode::User, reg::CPSR, frame.cpsr);
                    self.entry_pc = frame.entry_pc;
                    instructions_executed += frame.executed;
                    if let Some(sample) = self.sampler.retire(frame.executed) {
                        jit.sample(sample.location, sample.hotness);
                        check_deadline = true;
                    }
                    if exit == CompiledExit::GuestFault {
                        return Err(WieError::InvalidMemoryAccess(frame.fault_address));
                    }
                    interpret_one = exit == CompiledExit::InterpretOne;
                    lookup_entry = true;
                    continue;
                }
            }
            let recheck_after_step = interpret_one;
            interpret_one = false;
            lookup_entry = false;

            if self.sampler.remaining == 1 {
                self.sampler
                    .prepare(pc, cpsr, self.cpu.reg_get(Mode::User, 7), self.entry_pc, |address, buffer| {
                        self.mem.read_range(address, buffer.len(), buffer).is_ok()
                    });
            }

            let cache_invalidation = self.instruction_cache_invalidation(pc, cpsr);
            let mut arm32cpu_memory = self.mem.as_arm32cpu_memory();
            if !(self.cpu.step(&mut arm32cpu_memory)) {
                return Err(WieError::FatalError("Undefined instruction".into()));
            }
            if let Some(x) = arm32cpu_memory.memory_error() {
                return Err(WieError::InvalidMemoryAccess(x));
            }
            instructions_executed += 1;
            let next_pc = self.cpu.reg_get(Mode::User, reg::PC);
            let next_cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            if next_pc != pc.wrapping_add(if cpsr & 0x20 != 0 { 2 } else { 4 }) || (next_cpsr ^ cpsr) & 0x0100_003f != 0 {
                self.entry_pc = next_pc;
                lookup_entry = true;
            }
            if let Some(invalidation) = cache_invalidation {
                self.mem.invalidate_instruction_cache(invalidation);
                self.entry_pc = 0;
                lookup_entry = true;
            }
            lookup_entry |= recheck_after_step;
            if let Some(sample) = self.sampler.retire(1) {
                check_deadline = true;
                if let Some(jit) = &mut self.jit {
                    jit.sample(sample.location, sample.hotness);
                }
            }
        };

        Ok(EngineRunResult {
            stop_reason,
            instructions_executed,
        })
    }

    fn mark_entry(&mut self) {
        self.entry_pc = self.cpu.reg_get(Mode::User, reg::PC);
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
        if matches!(reg, ArmRegister::PC | ArmRegister::Cpsr) {
            self.entry_pc = 0;
        }
        if reg == ArmRegister::PC && value % 2 == 1 {
            self.cpu.reg_set(Mode::User, reg.into_armv4t(), value - 1);

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            self.cpu.reg_set(Mode::User, reg::CPSR, cpsr | (1 << 5)); // T bit

            return;
        }
        self.cpu.reg_set(Mode::User, reg.into_armv4t(), value);
    }

    fn reg_read(&self, reg: ArmRegister) -> u32 {
        self.cpu.reg_get(Mode::User, reg.into_armv4t())
    }

    fn mem_map(&mut self, address: u32, size: usize, _permission: MemoryPermission) {
        self.mem.map(address, size);
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        self.mem.write_range(address, data)
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        self.mem.read_range(address, size, result)
    }

    fn mem_read_until_nul(&mut self, address: u32) -> Result<Vec<u8>> {
        self.mem.read_until_nul(address)
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.mem.is_mapped(address, size)
    }

    fn set_profiling(&mut self, enabled: bool) {
        self.sampler.profiling = enabled;
    }

    fn take_profile(&mut self, force: bool) -> Vec<ProfileSample> {
        self.sampler.take_profile(force)
    }

    fn maintain(&mut self) {
        if let Some(jit) = &mut self.jit {
            jit.maintain(&self.mem);
        }
    }

    fn shutdown(&mut self) {
        self.closed = true;
        self.jit = None;
    }
}

struct MemoryAccess<'a> {
    memory: &'a mut EmulatedMemory,
    sampler: &'a mut Sampler,
}

impl ExecutionAccess for MemoryAccess<'_> {
    fn word_range(&mut self, address: u32, words: u32) -> Option<(&mut [u8], &mut [u8])> {
        if !address.is_multiple_of(4) {
            return None;
        }
        let start = address as usize / PAGE_SIZE;
        let offset = (address & PAGE_MASK) as usize;
        let len = words as usize * 4;
        let first_len = len.min(PAGE_SIZE - offset);
        if first_len == len {
            let page = self.memory.pages[start].as_mut()?;
            return Some((&mut page.bytes[offset..offset + len], &mut []));
        }
        let end = address.wrapping_add((words - 1) * 4) as usize / PAGE_SIZE;
        let (first, second) = if start < end {
            let (lower, upper) = self.memory.pages.split_at_mut(end);
            (lower[start].as_mut()?, upper[0].as_mut()?)
        } else {
            let (lower, upper) = self.memory.pages.split_at_mut(start);
            (upper[0].as_mut()?, lower[end].as_mut()?)
        };
        Some((&mut first.bytes[offset..], &mut second.bytes[..len - first_len]))
    }

    fn load(&mut self, address: u32, width: u32) -> AccessResult {
        if !address.is_multiple_of(width) {
            return AccessResult::InterpretOne;
        }
        let Some(page) = self.memory.pages[address as usize / PAGE_SIZE].as_ref() else {
            return AccessResult::InterpretOne;
        };
        let bytes = &page.bytes[(address & PAGE_MASK) as usize..];
        let value = match width {
            1 => u32::from(bytes[0]),
            2 => u32::from(u16::from_le_bytes([bytes[0], bytes[1]])),
            _ => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        };
        AccessResult::Complete(value)
    }

    fn store(&mut self, address: u32, width: u32, value: u32) -> AccessResult {
        if !address.is_multiple_of(width) {
            return AccessResult::InterpretOne;
        }
        let Some(page) = self.memory.pages[address as usize / PAGE_SIZE].as_mut() else {
            return AccessResult::InterpretOne;
        };
        let bytes = &mut page.bytes[(address & PAGE_MASK) as usize..];
        match width {
            1 => bytes[0] = value as u8,
            2 => bytes[..2].copy_from_slice(&(value as u16).to_le_bytes()),
            _ => bytes[..4].copy_from_slice(&value.to_le_bytes()),
        }
        AccessResult::Complete(0)
    }

    fn sample_prepare(&mut self, pc: u32, cpsr: u32, r7: u32, entry_pc: u32) {
        self.sampler.prepare(pc, cpsr, r7, entry_pc, |address, buffer| {
            self.memory.read_range(address, buffer.len(), buffer).is_ok()
        });
    }
}

impl ArmRegister {
    fn into_armv4t(self) -> u8 {
        match self {
            ArmRegister::R0 => 0,
            ArmRegister::R1 => 1,
            ArmRegister::R2 => 2,
            ArmRegister::R3 => 3,
            ArmRegister::R4 => 4,
            ArmRegister::R5 => 5,
            ArmRegister::R6 => 6,
            ArmRegister::R7 => 7,
            ArmRegister::R8 => 8,
            ArmRegister::SB => 9,
            ArmRegister::SL => 10,
            ArmRegister::FP => 11,
            ArmRegister::IP => 12,
            ArmRegister::SP => reg::SP,
            ArmRegister::LR => reg::LR,
            ArmRegister::PC => reg::PC,
            ArmRegister::Cpsr => reg::CPSR,
        }
    }
}

const TOTAL_MEMORY: u64 = 0x100000000;
const PAGE_SIZE: usize = 0x10000;
const PAGE_MASK: u32 = (PAGE_SIZE - 1) as _;

struct MemoryPage {
    bytes: Box<[u8; PAGE_SIZE]>,
    version: u64,
}

pub(crate) struct EmulatedMemory {
    pages: Box<[Option<MemoryPage>]>,
}

impl EmulatedMemory {
    fn new() -> Self {
        Self {
            pages: (0..TOTAL_MEMORY / PAGE_SIZE as u64).map(|_| None).collect::<Vec<_>>().into_boxed_slice(),
        }
    }

    fn as_arm32cpu_memory(&mut self) -> Arm32CpuMemory<'_> {
        Arm32CpuMemory::new(self)
    }

    fn map(&mut self, address: u32, size: usize) {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            let page_data = &mut self.pages[page as usize / PAGE_SIZE];
            if page_data.is_none() {
                *page_data = Some(MemoryPage {
                    bytes: Box::new([0; PAGE_SIZE]),
                    version: 0,
                });
            }
        }
    }

    fn read_range(&self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        let mut remaining_size = size;
        let mut current_address = address;

        while remaining_size > 0 {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE]
                .as_ref()
                .ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(remaining_size);

            result[size - remaining_size..size - remaining_size + available_bytes]
                .copy_from_slice(&page_data.bytes[offset..offset + available_bytes]);
            remaining_size -= available_bytes;
            current_address += available_bytes as u32;
        }

        Ok(size)
    }

    fn read_until_nul(&self, address: u32) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut cursor = address;
        loop {
            let page = self.pages[cursor as usize / PAGE_SIZE]
                .as_ref()
                .ok_or(WieError::InvalidMemoryAccess(cursor))?;
            let bytes = &page.bytes[(cursor & PAGE_MASK) as usize..];
            if let Ok(value) = CStr::from_bytes_until_nul(bytes) {
                result.extend_from_slice(value.to_bytes());
                return Ok(result);
            }
            result.extend_from_slice(bytes);
            cursor = cursor.wrapping_add(bytes.len() as u32);
        }
    }

    fn write_range(&mut self, address: u32, data: &[u8]) -> Result<()> {
        let mut current_address = address;
        let mut data_index = 0;

        while data_index < data.len() {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE]
                .as_mut()
                .ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(data.len() - data_index);

            page_data.bytes[offset..offset + available_bytes].copy_from_slice(&data[data_index..data_index + available_bytes]);
            page_data.version = page_data.version.wrapping_add(1);
            data_index += available_bytes;
            current_address += available_bytes as u32;
        }

        Ok(())
    }

    pub(crate) fn code_snapshot(&self, pc: u32) -> Option<(u32, Vec<u8>, CodePageStamp)> {
        let page = self.pages[pc as usize / PAGE_SIZE].as_ref()?;
        let base = pc & !0x3fff;
        let offset = (base & PAGE_MASK) as usize;
        Some((
            base,
            page.bytes[offset..offset + 0x4000].to_vec(),
            CodePageStamp {
                page: pc & !PAGE_MASK,
                version: page.version,
            },
        ))
    }

    pub(crate) fn code_is_current(&self, source: &[CodePageStamp]) -> bool {
        source.iter().all(|stamp| {
            self.pages[stamp.page as usize / PAGE_SIZE]
                .as_ref()
                .is_some_and(|page| page.version == stamp.version)
        })
    }

    fn invalidate_instruction_cache(&mut self, invalidation: InstructionCacheInvalidation) {
        match invalidation {
            InstructionCacheInvalidation::All => {
                for page in self.pages.iter_mut().flatten() {
                    page.version = page.version.wrapping_add(1);
                }
            }
            InstructionCacheInvalidation::Address(address) => {
                if let Some(page) = &mut self.pages[address as usize / PAGE_SIZE] {
                    page.version = page.version.wrapping_add(1);
                }
            }
        }
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        if self.pages[page_start as usize / PAGE_SIZE].is_none() {
            return false;
        }

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            if self.pages[page as usize / PAGE_SIZE].is_none() {
                return false;
            }
        }

        true
    }
}

struct Arm32CpuMemory<'a> {
    emulated_memory: &'a mut EmulatedMemory,
    memory_error: RefCell<Option<u32>>,
}

impl<'a> Arm32CpuMemory<'a> {
    fn new(emulated_memory: &'a mut EmulatedMemory) -> Self {
        Self {
            emulated_memory,
            memory_error: RefCell::new(None),
        }
    }

    fn memory_error(&self) -> Option<u32> {
        *self.memory_error.borrow()
    }

    fn get_page(&mut self, addr: u32) -> Option<&mut MemoryPage> {
        let page_address = addr & !PAGE_MASK;
        let page_data = self.emulated_memory.pages[page_address as usize / PAGE_SIZE].as_mut();

        if let Some(x) = page_data {
            Some(x)
        } else {
            *self.memory_error.borrow_mut() = Some(addr);
            None
        }
    }
}

impl Memory for Arm32CpuMemory<'_> {
    fn r8(&mut self, addr: u32) -> u8 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();

        data.bytes[offset as usize]
    }

    fn r16(&mut self, addr: u32) -> u16 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();

        (data.bytes[offset as usize] as u16) | ((data.bytes[offset as usize + 1] as u16) << 8)
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();
        (data.bytes[offset as usize] as u32)
            | ((data.bytes[offset as usize + 1] as u32) << 8)
            | ((data.bytes[offset as usize + 2] as u32) << 16)
            | ((data.bytes[offset as usize + 3] as u32) << 24)
    }

    fn w8(&mut self, addr: u32, val: u8) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data.bytes[offset as usize] = val;
    }

    fn w16(&mut self, addr: u32, val: u16) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data.bytes[offset as usize] = val as u8;
        data.bytes[offset as usize + 1] = (val >> 8) as u8;
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data.bytes[offset as usize] = val as u8;
        data.bytes[offset as usize + 1] = (val >> 8) as u8;
        data.bytes[offset as usize + 2] = (val >> 16) as u8;
        data.bytes[offset as usize + 3] = (val >> 24) as u8;
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, sync::Arc};
    use core::mem::size_of;

    use arm32_cpu::Memory;
    use spin::Mutex;
    use wie_arm_jit::{
        Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion,
    };

    use crate::engine::{ArmEngine, ArmRegister, EngineStopReason, MemoryPermission};

    use super::*;

    #[test]
    fn terminated_string_reads_preserve_memory_and_page_boundaries() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x10000, PAGE_SIZE, MemoryPermission::ReadWrite);
        engine.mem_write(0x10003, &[0xff, 0x80, 0, b'x']).unwrap();
        engine.mem_write(0x1fffc, b"end\0").unwrap();
        let before = engine.mem.pages[1].as_ref().unwrap().bytes.clone();
        let stamp = engine.mem.code_snapshot(0x10000).unwrap().2;
        assert_eq!(engine.mem_read_until_nul(0x10003).unwrap(), [0xff, 0x80]);
        assert!(engine.mem_read_until_nul(0x10005).unwrap().is_empty());
        assert_eq!(engine.mem_read_until_nul(0x1fffc).unwrap(), b"end");
        assert!(engine.mem_read_until_nul(0x1ffff).unwrap().is_empty());
        assert!(matches!(engine.mem_read_until_nul(0x20007), Err(WieError::InvalidMemoryAccess(0x20007))));
        assert!(engine.mem.code_is_current(&[stamp]));
        assert_eq!(engine.mem.pages[1].as_ref().unwrap().bytes, before);

        engine.mem_write(0x1ffff, b"x").unwrap();
        let before = engine.mem.pages[1].as_ref().unwrap().bytes.clone();
        let stamp = engine.mem.code_snapshot(0x10000).unwrap().2;
        assert!(matches!(engine.mem_read_until_nul(0x1fffc), Err(WieError::InvalidMemoryAccess(0x20000))));
        assert!(engine.mem.code_is_current(&[stamp]));
        assert_eq!(engine.mem.pages[1].as_ref().unwrap().bytes, before);

        engine.mem_map(0x20000, PAGE_SIZE, MemoryPermission::ReadWrite);
        engine.mem_write(0x20000, b"page\0ignored").unwrap();
        let second = engine.mem.pages[2].as_ref().unwrap().bytes.clone();
        let second_stamp = engine.mem.code_snapshot(0x20000).unwrap().2;
        assert_eq!(engine.mem_read_until_nul(0x1fffc).unwrap(), b"endxpage");
        assert!(engine.mem.code_is_current(&[stamp, second_stamp]));
        assert_eq!(engine.mem.pages[1].as_ref().unwrap().bytes, before);
        assert_eq!(engine.mem.pages[2].as_ref().unwrap().bytes, second);
    }

    #[test]
    fn terminated_string_reads_wrap_at_the_last_guest_page() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem.pages[0xffff] = Some(MemoryPage {
            bytes: Box::new([b'x'; PAGE_SIZE]),
            version: 7,
        });
        engine.mem.pages[0xffff].as_mut().unwrap().bytes[PAGE_SIZE - 1] = 0;
        assert!(engine.mem_read_until_nul(u32::MAX).unwrap().is_empty());
        assert_eq!(engine.mem_read_until_nul(u32::MAX - 1).unwrap(), b"x");
        engine.mem.pages[0xffff].as_mut().unwrap().bytes[PAGE_SIZE - 1] = b'x';
        assert!(matches!(engine.mem_read_until_nul(u32::MAX), Err(WieError::InvalidMemoryAccess(0))));

        engine.mem_map(0, PAGE_SIZE, MemoryPermission::ReadWrite);
        engine.mem_write(0, b"y\0ignored").unwrap();
        let before = engine.mem.pages[0].as_ref().unwrap().bytes.clone();
        let stamps = [0, 0xffff0000].map(|address| engine.mem.code_snapshot(address).unwrap().2);
        assert_eq!(engine.mem_read_until_nul(0).unwrap(), b"y");
        assert_eq!(engine.mem_read_until_nul(u32::MAX).unwrap(), b"xy");
        assert_eq!(engine.mem_read_until_nul(u32::MAX - 1).unwrap(), b"xxy");
        assert!(engine.mem.code_is_current(&stamps));
        assert_eq!(engine.mem.pages[0].as_ref().unwrap().bytes, before);
        assert!(engine.mem.pages[0xffff].as_ref().unwrap().bytes.iter().all(|&byte| byte == b'x'));
    }

    struct TestExecutor {
        completion: Option<CompileCompletion>,
        calls: Arc<Mutex<(u32, bool)>>,
        trap: bool,
    }

    impl CompiledExecutor for TestExecutor {
        fn submit(&mut self, request: &CompileRequest) -> Admission {
            self.completion = Some(CompileCompletion {
                session: request.session,
                request: request.request,
                result: Ok(CompiledArtifact {
                    regions: request
                        .regions
                        .iter()
                        .enumerate()
                        .map(|(slot, region)| CompiledRegion {
                            manifest: ManifestRegion {
                                entry: region.ir.entry,
                                source: region.source.clone(),
                                expected_old: region.expected_old,
                                export: format!("region_{slot}"),
                            },
                            handle: CompiledHandle {
                                slot: slot as u32,
                                generation: request.request,
                            },
                        })
                        .collect(),
                    encoded_size: 100,
                }),
            });
            Admission::Accepted
        }

        fn poll(&mut self) -> Option<CompileCompletion> {
            self.completion.take()
        }

        fn execute(
            &mut self,
            _: CompiledHandle,
            frame: &mut RunFrame,
            access: &mut dyn ExecutionAccess,
        ) -> core::result::Result<CompiledExit, String> {
            self.calls.lock().0 += 1;
            if self.trap {
                assert!(matches!(access.store(0x20000, 4, 42), AccessResult::Complete(0)));
                frame.regs[0] = 99;
                return Err("injected trap after store".into());
            }
            Ok(CompiledExit::InterpretOne)
        }

        fn retire(&mut self, _: &[CompiledHandle]) {}

        fn shutdown(&mut self) {
            self.calls.lock().1 = true;
            self.completion = None;
        }
    }

    #[test]
    fn interpreter_handoff_advances_once_and_traps_never_replay_or_commit_the_frame() {
        for trap in [false, true] {
            let calls = Arc::new(Mutex::new((0, false)));
            let mut engine = Arm32CpuEngine::new();
            engine.mem_map(0x1000, 6, MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 4, MemoryPermission::ReadWrite);
            engine.mem_write(0x1000, &[0x01, 0x30, 0x70, 0x47]).unwrap(); // add r0, #1; bx lr
            engine.reg_write(ArmRegister::Cpsr, 0x3f);
            engine.reg_write(ArmRegister::PC, 0x1001);
            engine.reg_write(ArmRegister::LR, 0x2000);
            let mut jit = Jit::new(
                1,
                Box::new(TestExecutor {
                    completion: None,
                    calls: calls.clone(),
                    trap,
                }),
            );
            let sampled_key = RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&engine.mem);
            engine.jit = Some(jit);
            let result = engine.run(0x2000, 10, None);
            assert_eq!(calls.lock().0, if trap { 1 } else { 2 });
            if trap {
                assert!(result.is_err());
                assert_eq!(engine.reg_read(ArmRegister::R0), 0);
                assert_eq!(engine.reg_read(ArmRegister::PC), 0x1000);
                let mut value = [0; 4];
                engine.mem_read(0x20000, 4, &mut value).unwrap();
                assert_eq!(u32::from_le_bytes(value), 42);
                assert!(calls.lock().1);
                assert!(engine.run(0x2000, 10, None).is_err());
                assert_eq!(calls.lock().0, 1);
            } else {
                let result = result.unwrap();
                assert!(matches!(result.stop_reason, EngineStopReason::End));
                assert_eq!(result.instructions_executed, 2);
                assert_eq!(engine.reg_read(ArmRegister::R0), 1);
                assert_eq!(engine.sampler.remaining, 1022);
            }
        }
    }

    #[test]
    fn code_versions_follow_host_publication_not_guest_stores() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, 0x10000);
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        memory.as_arm32cpu_memory().w8(0x10000, 42);
        memory.as_arm32cpu_memory().w16(0x10000, 42);
        memory.as_arm32cpu_memory().w32(0x10000, 42);
        assert!(memory.code_is_current(&[before]));
        assert!(memory.write_range(0x1ffff, &[1, 2]).is_err());
        assert!(!memory.code_is_current(&[before]));
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
        };
        assert!(matches!(access.store(0x10001, 4, 42), AccessResult::InterpretOne));
        assert!(access.memory.code_is_current(&[before]));
        assert!(matches!(access.store(0x20000, 4, 42), AccessResult::InterpretOne));
        assert!(matches!(access.store(0x10000, 2, 42), AccessResult::Complete(0)));
        assert!(access.memory.code_is_current(&[before]));
        access.memory.invalidate_instruction_cache(InstructionCacheInvalidation::Address(0x10000));
        assert!(!access.memory.code_is_current(&[before]));
    }

    #[test]
    fn memory_access_preserves_widths_boundaries_and_code_versions() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, PAGE_SIZE);
        memory.pages[0xffff] = Some(MemoryPage {
            bytes: Box::new([0; PAGE_SIZE]),
            version: 7,
        });
        let (_, _, first_page) = memory.code_snapshot(0x10000).unwrap();
        let (_, _, last_page) = memory.code_snapshot(0xffff_0000).unwrap();
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
        };
        for base in [0x10000, 0x1fffc, 0xffff_fffc] {
            for (width, expected) in [(1, 0xef), (2, 0xcdef), (4, 0x89ab_cdef)] {
                let offset = (base & PAGE_MASK) as usize;
                access.memory.pages[base as usize / PAGE_SIZE].as_mut().unwrap().bytes[offset..offset + 4].fill(0x55);
                assert!(matches!(access.load(base, 4), AccessResult::Complete(0x5555_5555)));
                let address = base + (4 - width);
                assert!(matches!(access.store(address, width, 0x89ab_cdef), AccessResult::Complete(0)));
                assert!(matches!(access.load(address, width), AccessResult::Complete(value) if value == expected));
                let mut bytes = [0x55; 4];
                bytes[4 - width as usize..].copy_from_slice(&[0xef, 0xcd, 0xab, 0x89][..width as usize]);
                assert_eq!(
                    access.memory.pages[base as usize / PAGE_SIZE].as_ref().unwrap().bytes[offset..offset + 4],
                    bytes
                );
            }
        }
        let first_bytes = access.memory.pages[1].as_ref().unwrap().bytes.clone();
        let last_bytes = access.memory.pages[0xffff].as_ref().unwrap().bytes.clone();
        for (address, width) in [
            (0x10001, 2),
            (0x10001, 4),
            (0x10002, 4),
            (0x10003, 4),
            (0x1ffff, 2),
            (0x1fffd, 4),
            (0x1fffe, 4),
            (0x1ffff, 4),
            (0xffff_ffff, 2),
            (0xffff_fffd, 4),
            (0xffff_fffe, 4),
            (0xffff_ffff, 4),
            (0x20000, 1),
            (0x20000, 2),
            (0x20000, 4),
        ] {
            assert!(
                matches!(access.load(address, width), AccessResult::InterpretOne),
                "{address:#x}, width={width}"
            );
            assert!(
                matches!(access.store(address, width, 42), AccessResult::InterpretOne),
                "{address:#x}, width={width}"
            );
        }
        assert_eq!(access.memory.pages[1].as_ref().unwrap().bytes, first_bytes);
        assert_eq!(access.memory.pages[0xffff].as_ref().unwrap().bytes, last_bytes);
        assert!(access.memory.code_is_current(&[first_page, last_page]));
    }

    #[test]
    fn word_range_admission_checks_mapping_without_reading_or_publishing() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, PAGE_SIZE);
        memory.pages[0xffff] = Some(MemoryPage {
            bytes: Box::new([0; PAGE_SIZE]),
            version: 7,
        });
        memory.write_range(0x10000, &42u32.to_le_bytes()).unwrap();
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
        };
        for (address, words, admitted) in [
            (0x10000, 16, true),
            (0x1ffc0, 16, true),
            (0x1fffc, 1, true),
            (0x1fffc, 2, false),
            (0x20000, 1, false),
            (0x10001, 1, false),
            (0x10002, 1, false),
            (0x10003, 1, false),
            (0xffff_fffc, 1, true),
            (0xffff_fffc, 2, false),
        ] {
            assert_eq!(access.word_range(address, words).is_some(), admitted, "{address:#x}, words={words}");
        }
        assert!(access.memory.code_is_current(&[before]));
        assert_eq!(access.memory.as_arm32cpu_memory().r32(0x10000), 42);
        assert_eq!(access.sampler.remaining, 1024);
        assert_eq!(access.sampler.sequence, 0);
        access.memory.map(0x20000, PAGE_SIZE);
        access.memory.map(0, PAGE_SIZE);
        assert!(access.word_range(0xffff_fffd, 1).is_none());
        let before = [0, 0x10000, 0x20000, 0xffff_0000].map(|address| access.memory.code_snapshot(address).unwrap().2);
        for (address, words, first_len, second_len) in [
            (0x10000, 1, 4, 0),
            (0x1ffc0, 16, 64, 0),
            (0x1fffc, 2, 4, 4),
            (0x1fffc, 16, 4, 60),
            (0xffff_ffc0, 16, 64, 0),
            (0xffff_fffc, 2, 4, 4),
            (0xffff_fffc, 16, 4, 60),
        ] {
            let (first, second) = access.word_range(address, words).unwrap();
            assert_eq!((first.len(), second.len()), (first_len, second_len));
            first.fill(0x11);
            second.fill(0x22);
            for offset in (0..words * 4).step_by(4) {
                let expected = if offset < first_len as u32 { 0x1111_1111 } else { 0x2222_2222 };
                assert_eq!(access.memory.as_arm32cpu_memory().r32(address.wrapping_add(offset)), expected);
            }
        }
        assert!(access.memory.code_is_current(&before));
        assert_eq!(access.sampler.remaining, 1024);
        assert_eq!(access.sampler.sequence, 0);
    }

    #[test]
    fn coprocessor_instruction_cache_maintenance_publishes_code() {
        for (opcode, operand, code_current, data_current) in [
            (0xee070f15u32, 0, false, false),
            (0xee070f17, 0, false, false),
            (0xee070f55, 0, false, false),
            (0xee070f35, 0x20020, true, false),
            (0xee070f35, 0x30000, true, true),
            (0xee070e15, 0, true, true),
            (0xee270f15, 0, true, true),
            (0xee170f15, 0, true, true),
            (0xee070f16, 0, true, true),
            (0xee070f95, 0, true, true),
        ] {
            let mut engine = Arm32CpuEngine::new();
            engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 4, MemoryPermission::ReadWrite);
            engine.mem_write(0x1000, &opcode.to_le_bytes()).unwrap();
            engine.reg_write(ArmRegister::Cpsr, 0x1f);
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.reg_write(ArmRegister::R0, operand);
            engine.sampler.remaining = 1;
            engine.set_profiling(true);
            let (_, _, code) = engine.mem.code_snapshot(0x1000).unwrap();
            let (_, _, data) = engine.mem.code_snapshot(0x20000).unwrap();
            let result = engine.run(0x1004, 1, None).unwrap();
            assert_eq!(result.instructions_executed, 1);
            assert!(matches!(result.stop_reason, EngineStopReason::End));
            assert_eq!(engine.mem.code_is_current(&[code]), code_current, "opcode={opcode:#x}");
            assert_eq!(engine.mem.code_is_current(&[data]), data_current, "opcode={opcode:#x}");
            assert!(engine.mem.pages[3].is_none());
            assert_eq!(engine.reg_read(ArmRegister::PC), 0x1004);
            assert_eq!(engine.reg_read(ArmRegister::Cpsr), 0x1f);
            assert_eq!(engine.sampler.sequence, 1);
            let samples = engine.take_profile(true);
            assert_eq!(samples.len(), 1);
            assert_eq!(samples[0].stack, [0x1000]);
        }
    }

    #[test]
    fn cache_maintenance_conditions_match_arm_branch_conditions() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 0x20, MemoryPermission::ReadWriteExecute);
        for condition in 0..15 {
            engine.mem_write(0x1000, &(0x0e070f15u32 | condition << 28).to_le_bytes()).unwrap();
            engine.mem_write(0x1010, &(0x0a000000u32 | condition << 28).to_le_bytes()).unwrap();
            for flags in 0..16 {
                let cpsr = 0x1f | flags << 28;
                engine.reg_write(ArmRegister::Cpsr, cpsr);
                engine.reg_write(ArmRegister::PC, 0x1010);
                engine.run(0x1018, 1, None).unwrap();
                let passed = engine.reg_read(ArmRegister::PC) == 0x1018;
                engine.reg_write(ArmRegister::PC, 0x1000);
                let (_, _, before) = engine.mem.code_snapshot(0x1000).unwrap();
                engine.run(0x1004, 1, None).unwrap();
                assert_eq!(!engine.mem.code_is_current(&[before]), passed, "condition={condition}, cpsr={cpsr:#x}");
            }
        }
    }

    #[test]
    fn cache_maintenance_reads_the_current_register_bank() {
        for (mode, cpsr, register) in [(Mode::Supervisor, 0x13, 13), (Mode::Fiq, 0x11, 8)] {
            let mut engine = Arm32CpuEngine::new();
            engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 4, MemoryPermission::ReadWrite);
            engine.mem_map(0x30000, 4, MemoryPermission::ReadWrite);
            engine
                .mem_write(0x1000, &(0xee070f35u32 | u32::from(register) << 12).to_le_bytes())
                .unwrap();
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.cpu.reg_set(Mode::User, register, 0x20000);
            engine.cpu.reg_set(mode, register, 0x30000);
            let (_, _, user_page) = engine.mem.code_snapshot(0x20000).unwrap();
            let (_, _, active_page) = engine.mem.code_snapshot(0x30000).unwrap();

            assert_eq!(engine.run(0x1004, 1, None).unwrap().instructions_executed, 1);
            assert!(engine.mem.code_is_current(&[user_page]), "mode={mode:?}");
            assert!(!engine.mem.code_is_current(&[active_page]), "mode={mode:?}");
            assert_eq!(engine.reg_read(ArmRegister::Cpsr), cpsr);
        }
    }

    #[test]
    fn expired_deadlines_preserve_state_and_existing_stop_priorities() {
        for thumb in [false, true] {
            for (pc, end, count) in [(0x1000, 0x2000, 2), (0x1000, 0x1000, 0), (0x1000, 0x2000, 0), (8, 8, 0)] {
                let mut engine = Arm32CpuEngine::new();
                let code: Vec<_> = if thumb {
                    [0x3001u16, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect()
                } else {
                    [0xe2800001u32, 0xe12fff1e].into_iter().flat_map(u32::to_le_bytes).collect()
                };
                engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, &code).unwrap();
                engine.reg_write(ArmRegister::Cpsr, if thumb { 0x3f } else { 0x1f });
                engine.reg_write(ArmRegister::PC, pc);
                engine.reg_write(ArmRegister::LR, 0x2000);
                engine.mark_entry();
                engine.sampler.remaining = 1;
                let result = engine.run(end, count, Some(Instant::now()));
                if pc < 0x1000 {
                    assert!(matches!(result, Err(WieError::InvalidMemoryAccess(8))));
                } else {
                    let result = result.unwrap();
                    assert_eq!(result.instructions_executed, 0);
                    assert!(matches!(
                        (result.stop_reason, pc == end, count == 0),
                        (EngineStopReason::End, true, _) | (EngineStopReason::Yield, false, true) | (EngineStopReason::Deadline, false, false)
                    ));
                }
                assert_eq!(engine.reg_read(ArmRegister::PC), pc);
                assert_eq!(engine.reg_read(ArmRegister::R0), 0);
                assert_eq!(engine.entry_pc, pc);
                assert_eq!(engine.sampler.remaining, 1);
                assert_eq!(engine.sampler.sequence, 0);
            }
        }

        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 2, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0x01, 0xdf]).unwrap();
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        let first = engine.run(0x2000, 1, None).unwrap();
        assert!(matches!(
            first.stop_reason,
            EngineStopReason::Svc {
                category: 1,
                lr: 0x1002,
                spsr: 0x3f
            }
        ));
        assert_eq!(first.instructions_executed, 1);
        let next = engine.run(8, 0, Some(Instant::now())).unwrap();
        assert!(matches!(
            next.stop_reason,
            EngineStopReason::Svc {
                category: 1,
                lr: 0x1002,
                spsr: 0x3f
            }
        ));
        assert_eq!(next.instructions_executed, 0);
    }

    #[test]
    fn running_loops_reach_deadlines_at_shared_sample_boundaries() {
        for thumb in [false, true] {
            let code: Vec<_> = if thumb {
                [0x3001u16, 0xe7fd].into_iter().flat_map(u16::to_le_bytes).collect()
            } else {
                [0xe2800001u32, 0xeafffffd].into_iter().flat_map(u32::to_le_bytes).collect()
            };
            let [mut timed, mut counted] = core::array::from_fn(|_| {
                let mut engine = Arm32CpuEngine::new();
                engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, &code).unwrap();
                engine.reg_write(ArmRegister::Cpsr, if thumb { 0x3f } else { 0x1f });
                engine.reg_write(ArmRegister::PC, 0x1000);
                engine
            });
            let result = timed
                .run(0x2000, u32::MAX, Some(Instant::now() + core::time::Duration::from_millis(8)))
                .unwrap();
            assert!(matches!(result.stop_reason, EngineStopReason::Deadline));
            // Entry can already be expired if the host thread was preempted.
            if result.instructions_executed != 0 {
                assert!(timed.sampler.sequence > 0);
            }
            let exact = counted
                .run(
                    0x2000,
                    result.instructions_executed,
                    Some(Instant::now() + core::time::Duration::from_secs(60)),
                )
                .unwrap();
            assert!(matches!(exact.stop_reason, EngineStopReason::Yield));
            assert_eq!(exact.instructions_executed, result.instructions_executed);
            assert_eq!(counted.reg_read(ArmRegister::R0), timed.reg_read(ArmRegister::R0));
            assert_eq!(counted.reg_read(ArmRegister::PC), timed.reg_read(ArmRegister::PC));
            assert_eq!(counted.sampler.sequence, timed.sampler.sequence);
            assert_eq!(counted.sampler.remaining, timed.sampler.remaining);
            let mut sample = Sampler::new();
            while sample.sequence < timed.sampler.sequence {
                sample.retire(sample.remaining);
            }
            assert_eq!(sample.remaining, timed.sampler.remaining);
        }
    }

    #[test]
    fn run_reports_executed_instructions_at_budget_and_return_boundaries() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0xc0, 0x46, 0xc0, 0x46, 0x70, 0x47]).unwrap(); // nop; nop; bx lr
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2000);

        for (budget, expected_count, at_end) in [(0, 0, false), (2, 2, false), (10, 1, true), (10, 0, true)] {
            let result = engine.run(0x2000, budget, None).unwrap();
            assert_eq!(result.instructions_executed, expected_count);
            assert!(matches!(
                (result.stop_reason, at_end),
                (EngineStopReason::End, true) | (EngineStopReason::Yield, false)
            ));
        }
    }

    #[test]
    fn entry_observation_survives_retirement_boundaries_but_not_external_context_writes() {
        for thumb in [false, true] {
            let mut engine = Arm32CpuEngine::new();
            let code: Vec<_> = if thumb {
                [0x46c0u16, 0x46c0, 0x4700, 0x46c0, 0x46c0, 0x4770]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect()
            } else {
                [0xe1a00000u32, 0xe1a00000, 0xe12fff10, 0xe1a00000, 0xe1a00000, 0xe12fff1e]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect()
            };
            let stride = if thumb { 2 } else { 4 };
            let cpsr = if thumb { 0x3f } else { 0x1f };
            let target = 0x1000 + 4 * stride;
            engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, &code).unwrap();
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, 0x1000 | u32::from(thumb));
            engine.reg_write(ArmRegister::R0, target | u32::from(thumb));
            engine.reg_write(ArmRegister::LR, 0x2000 | u32::from(thumb));
            assert_eq!(engine.entry_pc, 0);
            engine.mark_entry();
            engine.sampler.remaining = 1;

            for budget in [0, 1, 1] {
                assert_eq!(engine.run(0x2000, budget, None).unwrap().instructions_executed, budget);
                assert_eq!(engine.entry_pc, 0x1000);
                engine.reg_write(ArmRegister::R1, 7);
                assert_eq!(engine.entry_pc, 0x1000);
            }
            assert_eq!(engine.sampler.sequence, 1);
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            assert_eq!(engine.entry_pc, 0, "even an unchanged external CPSR invalidates the observation");
            assert_eq!(engine.run(0x2000, 1, None).unwrap().instructions_executed, 1);
            assert_eq!(engine.entry_pc, target, "the retired transfer establishes the new entry");
            engine.reg_write(ArmRegister::PC, target | u32::from(thumb));
            assert_eq!(engine.entry_pc, 0);
            assert_eq!(engine.run(0x2000, 1, None).unwrap().instructions_executed, 1);
            assert_eq!(engine.entry_pc, 0, "linear execution does not invent a context entry");
            assert_eq!(engine.run(0x2000, 1, None).unwrap().instructions_executed, 1);
            assert_eq!(engine.entry_pc, 0x2000);
        }
    }

    #[test]
    fn svc_samples_capture_the_retired_instruction_before_exception_entry() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 2, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0x01, 0xdf]).unwrap();
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.set_profiling(true);
        engine.sampler.remaining = 1;
        let result = engine.run(0x2000, 1, None).unwrap();
        assert!(matches!(result.stop_reason, EngineStopReason::Svc { category: 1, .. }));
        assert_eq!(result.instructions_executed, 1);
        let samples = engine.take_profile(true);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stack, [0x1000]);
        assert_eq!(samples[0].count, 1);
        assert_eq!(engine.sampler.sequence, 1);
        engine.shutdown();
        assert!(engine.run(0x2000, 1, None).is_err());
    }

    #[test]
    fn page_table_is_heap_allocated() {
        assert_eq!(size_of::<EmulatedMemory>(), size_of::<Box<[Option<Box<[u8; super::PAGE_SIZE]>>]>>());
    }

    #[test]
    fn test_memory_basic() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x1000);
        memory.map(0x11000, 0x1000);
        memory.map(0x20000, 0x10000);

        memory.write_range(0x10000, &[123; 0x1000]).unwrap();

        let mut buf = [0; 0x1000];
        memory.read_range(0x10000, 0x1000, &mut buf).unwrap();
        assert_eq!(buf, [123; 0x1000]);

        memory.write_range(0x10900, &[100; 0x1000]).unwrap();

        memory.read_range(0x10900, 0x1000, &mut buf).unwrap();
        assert_eq!(buf, [100; 0x1000]);

        let mut arm32cpu_memory = memory.as_arm32cpu_memory();

        let r8 = arm32cpu_memory.r8(0x10000);
        assert_eq!(r8, 123);

        let r16 = arm32cpu_memory.r16(0x10000);
        assert_eq!(r16, 123 | (123 << 8));

        let r32 = arm32cpu_memory.r32(0x10000);
        assert_eq!(r32, 123 | (123 << 8) | (123 << 16) | (123 << 24));

        arm32cpu_memory.w8(0x10000, 12);
        let r8 = arm32cpu_memory.r8(0x10000);
        assert_eq!(r8, 12);

        arm32cpu_memory.w16(0x10000, 0x1234);
        let r16 = arm32cpu_memory.r16(0x10000);
        assert_eq!(r16, 0x1234);

        arm32cpu_memory.w32(0x10000, 0x12345678);
        let r32 = arm32cpu_memory.r32(0x10000);
        assert_eq!(r32, 0x12345678);
    }

    #[test]
    fn test_memory_unmapped_read() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        let mut buf = [0; 0x1000];
        assert!(memory.read_range(0x1f500, 0x1000, &mut buf).is_err());
    }

    #[test]
    fn test_memory_unmapped_write() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        assert!(memory.write_range(0x1f500, &[12; 0x1000]).is_err());
    }
}
