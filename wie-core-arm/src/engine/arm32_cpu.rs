use alloc::{boxed::Box, format, string::String, vec::Vec};

use arm32_cpu::{Cpu, Memory, Mode, reg};

use wie_arm_jit_types::{
    AccessResult, CodeImage, CodePageStamp, CompiledArtifact, CompiledExit, CompiledHandle, ExecutionAccess, PreparationFuture, PreparationState,
    RegionKey, RunFrame,
};
use wie_backend::ProfileSample;
use wie_util::{Result, WieError};

use crate::{
    aot::{Aot, DummyExecutor},
    engine::{ArmEngine, ArmRegister, EngineRunResult, EngineStopReason, MemoryPermission},
};

use super::sampler::Sampler;

pub struct Arm32CpuEngine {
    cpu: Cpu,
    mem: EmulatedMemory,
    sampler: Sampler,
    aot: Option<Aot>,
    closed: bool,
}

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
            aot: Some(Aot::new(Box::new(DummyExecutor))),
            closed: false,
        }
    }

    pub fn with_backend() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let mut engine = Self::new();
            engine.aot = Some(Aot::new(Box::new(wie_core_arm_wasm::WasmExecutor::default())));
            engine
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::new()
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
    fn run(&mut self, end: u32, count: u32) -> Result<EngineRunResult> {
        if self.closed {
            return Err(WieError::FatalError("ARM core is shut down".into()));
        }
        let mut instructions_executed = 0;
        let mut interpret_one = false;
        let mut lookup_entry = true;
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

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            if lookup_entry
                && !interpret_one
                && cpsr & 0x0100_0000 == 0
                && let Some(aot) = &mut self.aot
            {
                let key = RegionKey {
                    pc,
                    thumb: cpsr & 0x20 != 0,
                    cpu_mode: (cpsr & 0x1f) as u8,
                };
                if let Some(handle) = aot.lookup(key, &self.mem) {
                    let mut frame = RunFrame {
                        regs: core::array::from_fn(|index| self.cpu.reg_get(Mode::User, index as u8)),
                        cpsr,
                        end,
                        budget_remaining: count - instructions_executed,
                        sample_remaining: self.sampler.remaining,
                        ..RunFrame::default()
                    };
                    let result = {
                        let (executor, resolve) = aot.execution_parts();
                        let mut access = MemoryAccess {
                            memory: &mut self.mem,
                            sampler: &mut self.sampler,
                            resolve: &resolve,
                        };
                        executor.execute(handle, &mut frame, &mut access)
                    };
                    let exit = match result {
                        Ok(exit) => exit,
                        Err(error) => {
                            self.shutdown();
                            return Err(WieError::FatalError(format!("ARM AOT execution failed: {error}")));
                        }
                    };
                    for (index, value) in frame.regs.into_iter().enumerate() {
                        self.cpu.reg_set(Mode::User, index as u8, value);
                    }
                    self.cpu.reg_set(Mode::User, reg::CPSR, frame.cpsr);
                    instructions_executed += frame.executed;
                    self.sampler.retire(frame.executed);
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
                self.sampler.prepare(pc, self.cpu.reg_get(Mode::User, 7), |address, buffer| {
                    self.mem.read_range(address, buffer.len(), buffer).is_ok()
                });
            }

            let cache_invalidation = self.instruction_cache_invalidation(pc, cpsr);
            let mut arm32cpu_memory = self.mem.as_arm32cpu_memory();
            if !(self.cpu.step(&mut arm32cpu_memory)) {
                return Err(WieError::FatalError("Undefined instruction".into()));
            }
            if let Some(x) = arm32cpu_memory.memory_error {
                return Err(WieError::InvalidMemoryAccess(x));
            }
            instructions_executed += 1;
            let next_pc = self.cpu.reg_get(Mode::User, reg::PC);
            let next_cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            if next_pc != pc.wrapping_add(if cpsr & 0x20 != 0 { 2 } else { 4 }) || (next_cpsr ^ cpsr) & 0x0100_003f != 0 {
                lookup_entry = true;
            }
            if let Some(invalidation) = cache_invalidation {
                self.mem.invalidate_instruction_cache(invalidation);
                lookup_entry = true;
            }
            lookup_entry |= recheck_after_step;
            self.sampler.retire(1);
        };

        Ok(EngineRunResult {
            stop_reason,
            instructions_executed,
        })
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
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

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.mem.is_mapped(address, size)
    }

    fn set_profiling(&mut self, enabled: bool) {
        self.sampler.profiling = enabled;
    }

    fn take_profile(&mut self, force: bool) -> Vec<ProfileSample> {
        self.sampler.take_profile(force)
    }

    fn record_image(&mut self, address: u32, size: usize) {
        if let Some(aot) = &mut self.aot {
            aot.record_image(address, size);
        }
    }

    fn begin_preparation(&mut self) -> Result<Option<PreparationFuture>> {
        if let Some(aot) = &mut self.aot {
            #[cfg(target_arch = "wasm32")]
            let now = wie_core_arm_wasm::now;
            #[cfg(not(target_arch = "wasm32"))]
            let now = || 0.0;
            return aot.begin(&self.mem, now);
        }
        Ok(None)
    }

    fn preparation_state(&self) -> PreparationState {
        self.aot.as_ref().map_or(PreparationState::Ready, |aot| aot.state)
    }

    fn finish_preparation(&mut self, result: core::result::Result<CompiledArtifact, String>) {
        if let Some(aot) = &mut self.aot {
            #[cfg(target_arch = "wasm32")]
            let now = wie_core_arm_wasm::now;
            #[cfg(not(target_arch = "wasm32"))]
            let now = || 0.0;
            aot.finish(result, &self.mem, now);
        }
    }

    fn shutdown(&mut self) {
        self.closed = true;
        self.aot = None;
    }
}

struct MemoryAccess<'a> {
    memory: &'a mut EmulatedMemory,
    sampler: &'a mut Sampler,
    resolve: &'a dyn Fn(RegionKey, &EmulatedMemory) -> Option<CompiledHandle>,
}

impl ExecutionAccess for MemoryAccess<'_> {
    fn resolve(&self, pc: u32, cpsr: u32) -> Option<CompiledHandle> {
        if cpsr & 0x0100_0000 != 0 {
            return None;
        }
        (self.resolve)(
            RegionKey {
                pc,
                thumb: cpsr & 0x20 != 0,
                cpu_mode: (cpsr & 0x1f) as u8,
            },
            self.memory,
        )
    }
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
            2 => u32::from(u16::from_le_bytes(bytes.as_chunks::<2>().0[0])),
            _ => u32::from_le_bytes(bytes.as_chunks::<4>().0[0]),
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

    fn sample_prepare(&mut self, pc: u32, r7: u32) {
        self.sampler
            .prepare(pc, r7, |address, buffer| self.memory.read_range(address, buffer.len(), buffer).is_ok());
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
            current_address = current_address.wrapping_add(available_bytes as u32);
        }

        Ok(size)
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

    pub(crate) fn code_image(&self, address: u32, size: usize) -> Result<CodeImage> {
        let mut bytes = alloc::vec![0; size];
        self.read_range(address, size, &mut bytes)?;
        let source = (u64::from(address & !PAGE_MASK)..u64::from(address) + size as u64)
            .step_by(PAGE_SIZE)
            .map(|page| CodePageStamp {
                page: page as u32,
                version: self.pages[page as usize / PAGE_SIZE].as_ref().unwrap().version,
            })
            .collect();
        Ok(CodeImage { address, bytes, source })
    }

    pub(crate) fn code_is_current(&self, source: &[CodePageStamp]) -> bool {
        source.iter().all(|stamp| {
            self.pages[stamp.page as usize / PAGE_SIZE]
                .as_ref()
                .is_some_and(|page| page.version == stamp.version)
        })
    }

    pub(crate) fn validate_code(&self, source: &mut [CodePageStamp], source_bytes: &[(u32, Vec<u8>)]) -> bool {
        if self.code_is_current(source) {
            return true;
        }
        // A cache flush publishes writes, but unchanged instructions still match their translation.
        for (address, bytes) in source_bytes {
            let mut address = *address;
            let mut remaining = bytes.as_slice();
            while !remaining.is_empty() {
                let Some(page) = &self.pages[address as usize / PAGE_SIZE] else {
                    return false;
                };
                let offset = (address & PAGE_MASK) as usize;
                let size = (PAGE_SIZE - offset).min(remaining.len());
                if page.bytes[offset..offset + size] != remaining[..size] {
                    return false;
                }
                remaining = &remaining[size..];
                address = address.wrapping_add(size as u32);
            }
        }
        for stamp in source {
            stamp.version = self.pages[stamp.page as usize / PAGE_SIZE].as_ref().unwrap().version;
        }
        true
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
    memory_error: Option<u32>,
}

impl<'a> Arm32CpuMemory<'a> {
    fn new(emulated_memory: &'a mut EmulatedMemory) -> Self {
        Self {
            emulated_memory,
            memory_error: None,
        }
    }

    fn get_page(&mut self, addr: u32) -> Option<&mut MemoryPage> {
        let page_address = addr & !PAGE_MASK;
        let page_data = self.emulated_memory.pages[page_address as usize / PAGE_SIZE].as_mut();

        if let Some(x) = page_data {
            Some(x)
        } else {
            self.memory_error = Some(addr);
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

        u16::from_le_bytes(data.bytes[offset as usize..offset as usize + 2].try_into().unwrap())
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();
        u32::from_le_bytes(data.bytes[offset as usize..offset as usize + 4].try_into().unwrap())
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

        data.bytes[offset as usize..offset as usize + 2].copy_from_slice(&val.to_le_bytes());
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data.bytes[offset as usize..offset as usize + 4].copy_from_slice(&val.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, sync::Arc};
    use core::{mem::size_of, task::Poll};

    use arm32_cpu::Memory;
    use spin::Mutex;
    use wie_arm_jit_types::{CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion};

    use crate::engine::{ArmEngine, ArmRegister, EngineStopReason, MemoryPermission};

    use super::*;

    #[test]
    fn memory_reads_wrap_at_the_last_guest_page() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem.pages[0xffff] = Some(MemoryPage {
            bytes: Box::new([b'x'; PAGE_SIZE]),
            version: 0,
        });
        engine.mem_map(0, PAGE_SIZE, MemoryPermission::ReadWrite);
        engine.mem_write(0, b"y").unwrap();
        let mut bytes = [0; 2];
        assert_eq!(engine.mem_read(u32::MAX, 2, &mut bytes).unwrap(), 2);
        assert_eq!(&bytes, b"xy");
    }

    #[derive(Default)]
    struct Responses {
        requests: Vec<(Vec<wie_arm_jit_types::CompileRegion>, f64)>,
        ready: Option<futures::channel::oneshot::Sender<core::result::Result<CompiledArtifact, String>>>,
        retired: Vec<CompiledHandle>,
        shutdowns: usize,
    }

    struct DeferredExecutor(Arc<Mutex<Responses>>);

    impl CompiledExecutor for DeferredExecutor {
        fn prepare(&mut self, request: CompileRequest, deadline_ms: f64) -> PreparationFuture {
            let (sender, receiver) = futures::channel::oneshot::channel();
            let mut state = self.0.lock();
            state.requests.push((request.regions.flatten().collect(), deadline_ms));
            state.ready = Some(sender);
            Box::pin(async move { receiver.await.unwrap_or_else(|_| Err("preparation cancelled".into())) })
        }

        fn execute(&mut self, _: CompiledHandle, _: &mut RunFrame, _: &mut dyn ExecutionAccess) -> core::result::Result<CompiledExit, String> {
            unreachable!("preparation tests do not execute compiled code")
        }

        fn retire(&mut self, handles: &[CompiledHandle]) {
            self.0.lock().retired.extend_from_slice(handles);
        }

        fn shutdown(&mut self) {
            let mut state = self.0.lock();
            state.shutdowns += 1;
            state.ready = None;
        }
    }

    fn artifact(request: &[wie_arm_jit_types::CompileRegion]) -> CompiledArtifact {
        CompiledArtifact {
            encoded_size: 100,
            regions: request
                .iter()
                .filter(|region| region.ir.entry.thumb)
                .take(1)
                .enumerate()
                .map(|(slot, region)| CompiledRegion {
                    manifest: ManifestRegion {
                        entry: region.ir.entry,
                        instruction_pcs: region
                            .ir
                            .blocks
                            .iter()
                            .flat_map(|block| &block.instructions)
                            .take(2)
                            .map(|instruction| instruction.pc)
                            .collect(),
                        source: region.source.clone(),
                        source_bytes: region.source_bytes.clone(),
                    },
                    handle: CompiledHandle { slot: slot as u32 },
                })
                .collect(),
        }
    }

    #[test]
    fn preparation_snapshots_final_file_bytes_without_bss_or_duplicate_ranges() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut aot = Aot::new(Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        memory.map(0x1000, 0x10000);
        memory.write_range(0x3ffe, &[1, 0x20, 2, 0x21, 0x70, 0x47]).unwrap();
        aot.record_image(0x3ffe, 6);
        aot.record_image(0x4000, 2);
        memory.write_range(0x4000, &[7, 0x21]).unwrap();
        let _preparation = aot.begin(&memory, || 200.0).unwrap().unwrap();

        let state = responses.lock();
        assert_eq!(state.requests.len(), 1);
        let (request, deadline) = &state.requests[0];
        assert_eq!(*deadline, 10_200.0);
        let instructions: Vec<_> = request
            .iter()
            .filter(|region| region.ir.entry.thumb)
            .flat_map(|region| &region.ir.blocks)
            .flat_map(|block| &block.instructions)
            .collect();
        assert_eq!(
            instructions.iter().map(|instruction| instruction.pc).collect::<Vec<_>>(),
            [0x3ffe, 0x4000, 0x4002]
        );
        for (instruction, immediate) in instructions.iter().zip([1, 7]) {
            assert!(
                matches!(instruction.operation, wie_arm_jit_types::ir::Operation::Alu { right: wie_arm_jit_types::ir::Operand { value: wie_arm_jit_types::ir::Value::Immediate(value), .. }, .. } if value == immediate)
            );
        }
        assert!(request.iter().all(|region| memory.code_is_current(&region.source)));
        assert_eq!(aot.state, PreparationState::Preparing);
        drop(state);
        assert!(aot.begin(&memory, || 300.0).unwrap().is_none());
        assert_eq!(responses.lock().requests.len(), 1);
        assert_eq!(aot.state, PreparationState::Preparing);
    }

    #[test]
    fn installed_coverage_is_exact_and_published_writes_never_recompile() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut aot = Aot::new(Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        memory.map(0x1000, 0x10000);
        memory.write_range(0x1000, &[1, 0x30, 0x70, 0x47, 0, 0]).unwrap();
        aot.record_image(0x1000, 6);
        let preparation = aot.begin(&memory, || 0.0).unwrap().unwrap();
        let compiled = artifact(&responses.lock().requests[0].0);
        assert!(responses.lock().ready.take().unwrap().send(Ok(compiled)).is_ok());
        aot.finish(futures::executor::block_on(preparation), &memory, || 1.0);
        assert_eq!(aot.state, PreparationState::Ready);
        let key = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        assert_eq!(aot.lookup(key, &memory), Some(CompiledHandle { slot: 0 }));
        assert_eq!(aot.lookup(RegionKey { pc: 0x1002, ..key }, &memory), Some(CompiledHandle { slot: 0 }));
        for wrong in [
            RegionKey { pc: 0x1001, ..key },
            RegionKey { pc: 0x1004, ..key },
            RegionKey { thumb: false, ..key },
            RegionKey { cpu_mode: 0x10, ..key },
            RegionKey { cpu_mode: 0x13, ..key },
        ] {
            assert_eq!(aot.lookup(wrong, &memory), None);
        }
        memory.as_arm32cpu_memory().w16(0x1000, 0x4770);
        assert_eq!(aot.lookup(key, &memory), Some(CompiledHandle { slot: 0 }));
        {
            let (_, resolve) = aot.execution_parts();
            assert_eq!(resolve(key, &memory), Some(CompiledHandle { slot: 0 }));
            assert_eq!(resolve(RegionKey { cpu_mode: 0x13, ..key }, &memory), None);
            memory.invalidate_instruction_cache(InstructionCacheInvalidation::Address(0x1000));
            assert_eq!(resolve(key, &memory), None);
            assert!(responses.lock().retired.is_empty());
        }
        assert_eq!(aot.lookup(key, &memory), None);
        assert_eq!(aot.lookup(RegionKey { pc: 0x1002, ..key }, &memory), None);
        assert_eq!(responses.lock().retired, [CompiledHandle { slot: 0 }]);
        memory.write_range(0x1000, &[0, 0]).unwrap();
        aot.record_image(0x1000, 2);
        assert!(aot.begin(&memory, || 2.0).unwrap().is_none());
        assert_eq!(responses.lock().requests.len(), 1);
    }

    #[test]
    fn cache_flushes_preserve_unchanged_instructions_but_retire_modified_code() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut aot = Aot::new(Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        memory.map(0x1000, 0x1000);
        memory.write_range(0x1000, &[1, 0x30, 0x70, 0x47]).unwrap();
        aot.record_image(0x1000, 4);
        let preparation = aot.begin(&memory, || 0.0).unwrap().unwrap();
        let compiled = artifact(&responses.lock().requests[0].0);
        assert!(responses.lock().ready.take().unwrap().send(Ok(compiled)).is_ok());
        aot.finish(futures::executor::block_on(preparation), &memory, || 1.0);
        let key = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let handle = Some(CompiledHandle { slot: 0 });

        memory.as_arm32cpu_memory().w16(0x1080, 42);
        memory.invalidate_instruction_cache(InstructionCacheInvalidation::All);
        assert_eq!(aot.lookup(key, &memory), handle);
        memory.write_range(0x1082, &[7]).unwrap();
        {
            let (_, resolve) = aot.execution_parts();
            assert_eq!(resolve(key, &memory), handle);
        }
        assert!(responses.lock().retired.is_empty());

        memory.as_arm32cpu_memory().w16(0x1000, 0x3002);
        assert_eq!(aot.lookup(key, &memory), handle);
        memory.invalidate_instruction_cache(InstructionCacheInvalidation::Address(0x1000));
        assert_eq!(aot.lookup(key, &memory), None);
        assert_eq!(aot.lookup(key, &memory), None);
        assert_eq!(responses.lock().retired, [CompiledHandle { slot: 0 }]);
        assert_eq!(responses.lock().requests.len(), 1);
    }

    #[test]
    fn failed_stale_cancelled_and_timed_out_preparations_do_not_install_late_results() {
        for cause in ["failed", "stale", "cancelled", "timeout", "installation-timeout"] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut aot = Aot::new(Box::new(DeferredExecutor(responses.clone())));
            let mut memory = EmulatedMemory::new();
            memory.map(0x1000, 0x1000);
            aot.record_image(0x1000, 4);
            let preparation = aot.begin(&memory, || 0.0).unwrap().unwrap();
            let compiled = artifact(&responses.lock().requests[0].0);
            let result = if cause == "failed" { Err("compile failed".into()) } else { Ok(compiled) };
            assert!(responses.lock().ready.take().unwrap().send(result).is_ok());
            if cause == "stale" {
                memory.write_range(0x1000, &[1]).unwrap();
            } else if cause == "cancelled" {
                aot.shutdown();
            }
            let time = core::cell::Cell::new(if cause == "timeout" { 10_000.0 } else { 9_999.0 });
            aot.finish(futures::executor::block_on(preparation), &memory, || {
                let value = time.get();
                if cause == "installation-timeout" {
                    time.set(10_000.0);
                }
                value
            });
            assert_eq!(aot.state, PreparationState::Ready, "{cause}");
            let key = RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            assert_eq!(aot.lookup(key, &memory), None, "{cause}");
            let late = artifact(&responses.lock().requests[0].0);
            aot.finish(Ok(late), &memory, || 10_001.0);
            assert_eq!(aot.lookup(key, &memory), None, "{cause}");
            assert_eq!(responses.lock().requests.len(), 1);
        }
    }

    struct TestExecutor {
        calls: Arc<Mutex<(u32, bool)>>,
        trap: bool,
    }

    impl CompiledExecutor for TestExecutor {
        fn prepare(&mut self, request: CompileRequest, _: f64) -> PreparationFuture {
            let compiled = artifact(&request.regions.flatten().collect::<Vec<_>>());
            Box::pin(async move { Ok(compiled) })
        }

        fn execute(
            &mut self,
            handle: CompiledHandle,
            frame: &mut RunFrame,
            access: &mut dyn ExecutionAccess,
        ) -> core::result::Result<CompiledExit, String> {
            self.calls.lock().0 += 1;
            assert_eq!(access.resolve(frame.regs[15], frame.cpsr), Some(handle));
            assert_eq!(access.resolve(frame.regs[15], frame.cpsr | 0x0100_0000), None);
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
            let mut aot = Aot::new(Box::new(TestExecutor { calls: calls.clone(), trap }));
            aot.record_image(0x1000, 4);
            let preparation = aot.begin(&engine.mem, || 0.0).unwrap().unwrap();
            aot.finish(futures::executor::block_on(preparation), &engine.mem, || 1.0);
            engine.aot = Some(aot);
            let result = engine.run(0x2000, 10);
            assert_eq!(calls.lock().0, if trap { 1 } else { 2 });
            if trap {
                assert!(result.is_err());
                assert_eq!(engine.reg_read(ArmRegister::R0), 0);
                assert_eq!(engine.reg_read(ArmRegister::PC), 0x1000);
                let mut value = [0; 4];
                engine.mem_read(0x20000, 4, &mut value).unwrap();
                assert_eq!(u32::from_le_bytes(value), 42);
                assert!(calls.lock().1);
                assert!(engine.run(0x2000, 10).is_err());
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
    fn preparation_waits_without_running_image_code_and_shutdown_does_not_resume_it() {
        for cancel in [false, true] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut engine = Arm32CpuEngine::new();
            engine.aot = Some(Aot::new(Box::new(DeferredExecutor(responses.clone()))));
            let mut core = crate::ArmCore::new(false, None).unwrap();
            core.inner.lock().engine = Box::new(engine);
            core.load(&[0x01, 0x30, 0x70, 0x47], 0x1000, 0x1000).unwrap();
            let controller = core.clone();
            let mut execution: core::pin::Pin<Box<dyn Future<Output = Result<u32>> + Send>> = Box::pin(async move {
                core.prepare_execution().await?;
                core.run_function(0x1001, &[]).await
            });
            let waker = futures::task::noop_waker();
            let mut context = core::task::Context::from_waker(&waker);
            assert!(execution.as_mut().poll(&mut context).is_pending());
            assert!(controller.is_preparing());
            assert_eq!(controller.inner.try_lock().unwrap().engine.reg_read(ArmRegister::R0), 0);
            if cancel {
                controller.shutdown();
                assert!(matches!(execution.as_mut().poll(&mut context), Poll::Ready(Err(_))));
            } else {
                assert!(
                    responses
                        .lock()
                        .ready
                        .take()
                        .unwrap()
                        .send(Ok(CompiledArtifact {
                            regions: Vec::new(),
                            encoded_size: 0
                        }))
                        .is_ok()
                );
                assert!(controller.is_preparing());
                assert!(matches!(execution.as_mut().poll(&mut context), Poll::Ready(Ok(1))));
                assert!(!controller.is_preparing());
            }
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_dummy_decodes_without_compilation_and_resumes_the_loader() {
        let steps = Arc::new(core::sync::atomic::AtomicUsize::new(0));
        let observed = steps.clone();
        let request = CompileRequest {
            images: Arc::from([]),
            regions: Box::new((0..3).map(move |_| {
                observed.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                None
            })),
        };
        let compiled = futures::executor::block_on(DummyExecutor.prepare(request, 0.0)).unwrap();
        assert_eq!(steps.load(core::sync::atomic::Ordering::Relaxed), 3);
        assert!(compiled.regions.is_empty());
        assert_eq!(compiled.encoded_size, 0);

        let mut core = crate::ArmCore::new(false, None).unwrap();
        assert!(core.is_preparing());
        core.load(&[42, 0x20, 0x70, 0x47], 0x1000, 0x1000).unwrap();
        futures::executor::block_on(core.prepare_execution()).unwrap();
        assert!(!core.is_preparing());
        assert_eq!(futures::executor::block_on(core.run_function::<u32>(0x1001, &[])).unwrap(), 42);
    }

    #[test]
    fn different_cpu_modes_and_jazelle_do_not_enter_system_translations() {
        for cpsr in [0x3f, 0x30, 0x33, 0x0100_003f] {
            let calls = Arc::new(Mutex::new((0, false)));
            let mut engine = Arm32CpuEngine::new();
            engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, &[0x01, 0x30, 0xc0, 0x46]).unwrap();
            let mut aot = Aot::new(Box::new(TestExecutor {
                calls: calls.clone(),
                trap: false,
            }));
            aot.record_image(0x1000, 4);
            let preparation = aot.begin(&engine.mem, || 0.0).unwrap().unwrap();
            aot.finish(futures::executor::block_on(preparation), &engine.mem, || 1.0);
            engine.aot = Some(aot);
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, 0x1001);
            assert_eq!(engine.run(0x1002, 1).unwrap().instructions_executed, 1);
            assert_eq!(engine.reg_read(ArmRegister::R0), 1);
            assert_eq!(calls.lock().0, u32::from(cpsr == 0x3f));
        }
    }

    #[test]
    fn published_code_validation_checks_cross_page_bytes_and_ignores_gaps() {
        let mut memory = EmulatedMemory::new();
        memory.map(0, 0x20000);
        memory.write_range(0xfffe, &[1, 2, 3, 4]).unwrap();
        memory.write_range(0x10010, &[5, 6]).unwrap();
        let image = memory.code_image(0xfffe, 4).unwrap();
        let mut source = image.source;
        let bytes = [(image.address, image.bytes), (0x10010, alloc::vec![5, 6])];
        memory.invalidate_instruction_cache(InstructionCacheInvalidation::All);
        memory.write_range(0x10008, &[42]).unwrap();
        assert!(memory.validate_code(&mut source, &bytes));
        assert!(memory.code_is_current(&source));
        memory.write_range(0x10001, &[7]).unwrap();
        assert!(!memory.validate_code(&mut source, &bytes));
        assert!(!memory.code_is_current(&source));
    }

    #[test]
    fn code_versions_follow_host_publication_not_guest_stores() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, 0x10000);
        let before = memory.code_image(0x10000, 1).unwrap().source[0];
        memory.as_arm32cpu_memory().w8(0x10000, 42);
        memory.as_arm32cpu_memory().w16(0x10000, 42);
        memory.as_arm32cpu_memory().w32(0x10000, 42);
        assert!(memory.code_is_current(&[before]));
        assert!(memory.write_range(0x1ffff, &[1, 2]).is_err());
        assert!(!memory.code_is_current(&[before]));
        let before = memory.code_image(0x10000, 1).unwrap().source[0];
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
            resolve: &|_, _| None,
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
        let first_page = memory.code_image(0x10000, 1).unwrap().source[0];
        let last_page = memory.code_image(0xffff_0000, 1).unwrap().source[0];
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
            resolve: &|_, _| None,
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
        let before = memory.code_image(0x10000, 1).unwrap().source[0];
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
            resolve: &|_, _| None,
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
        let before = [0, 0x10000, 0x20000, 0xffff_0000].map(|address| access.memory.code_image(address, 1).unwrap().source[0]);
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
            let code = engine.mem.code_image(0x1000, 1).unwrap().source[0];
            let data = engine.mem.code_image(0x20000, 1).unwrap().source[0];
            let result = engine.run(0x1004, 1).unwrap();
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
                engine.run(0x1018, 1).unwrap();
                let passed = engine.reg_read(ArmRegister::PC) == 0x1018;
                engine.reg_write(ArmRegister::PC, 0x1000);
                let before = engine.mem.code_image(0x1000, 1).unwrap().source[0];
                engine.run(0x1004, 1).unwrap();
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
            let user_page = engine.mem.code_image(0x20000, 1).unwrap().source[0];
            let active_page = engine.mem.code_image(0x30000, 1).unwrap().source[0];

            assert_eq!(engine.run(0x1004, 1).unwrap().instructions_executed, 1);
            assert!(engine.mem.code_is_current(&[user_page]), "mode={mode:?}");
            assert!(!engine.mem.code_is_current(&[active_page]), "mode={mode:?}");
            assert_eq!(engine.reg_read(ArmRegister::Cpsr), cpsr);
        }
    }

    #[test]
    fn run_reports_executed_instructions_at_budget_and_return_boundaries() {
        let mut engine = Arm32CpuEngine::new();
        assert!(engine.aot.is_some());
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0xc0, 0x46, 0xc0, 0x46, 0x70, 0x47]).unwrap(); // nop; nop; bx lr
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2000);

        for (budget, expected_count, at_end) in [(0, 0, false), (2, 2, false), (10, 1, true), (10, 0, true)] {
            let result = engine.run(0x2000, budget).unwrap();
            assert_eq!(result.instructions_executed, expected_count);
            assert!(matches!(
                (result.stop_reason, at_end),
                (EngineStopReason::End, true) | (EngineStopReason::Yield, false)
            ));
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
        let result = engine.run(0x2000, 1).unwrap();
        assert!(matches!(result.stop_reason, EngineStopReason::Svc { category: 1, .. }));
        assert_eq!(result.instructions_executed, 1);
        let samples = engine.take_profile(true);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stack, [0x1000]);
        assert_eq!(samples[0].count, 1);
        assert_eq!(engine.sampler.sequence, 1);
        engine.shutdown();
        assert!(engine.run(0x2000, 1).is_err());
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

        let mut access = memory.as_arm32cpu_memory();
        assert_eq!(access.r32(0x20000), 0);
        assert_eq!(access.memory_error, Some(0x20000));
    }

    #[test]
    fn test_memory_unmapped_write() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        assert!(memory.write_range(0x1f500, &[12; 0x1000]).is_err());

        let mut access = memory.as_arm32cpu_memory();
        access.w32(0x20000, 12);
        assert_eq!(access.memory_error, Some(0x20000));
    }
}
