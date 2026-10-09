use alloc::{boxed::Box, format, vec::Vec};

use arm32_cpu::{Cpu, Memory, Mode, reg};
use hashbrown::HashMap;

use wie_arm_jit_types::{
    CodeImage, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, MemoryPage, PreparationFuture, PreparationState,
    RegionKey, RunFrame,
};
use wie_util::{Result, WieError};

use crate::{
    aot::Aot,
    engine::{ArmEngine, ArmRegister, EngineRunResult, EngineStopReason, MemoryPermission},
};

pub struct Arm32CpuEngine {
    cpu: Cpu,
    mem: EmulatedMemory,
    aot: Option<Aot>,
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
            aot: None,
        }
    }

    pub fn with_backend(executor: Option<Box<dyn CompiledExecutor>>) -> Self {
        Self {
            aot: executor.map(Aot::new),
            ..Self::new()
        }
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
        let page = self.mem.pages[pc as usize / PAGE_SIZE].bytes.as_ref()?;
        let offset = (pc & PAGE_MASK & !3) as usize;
        let instruction = u32::from_le_bytes(core::array::from_fn(|index| page[offset + index])).rotate_right((pc & 3) * 8);
        // ARM926EJ-S CP15 c7: I-cache all/MVA/set-way and combined I/D invalidation.
        if instruction & 0x0fff_0f10 != 0x0e07_0f10 {
            return None;
        }
        // Matching cache-maintenance opcodes invalidate regardless of NZCV.
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
        if let Some(aot) = &mut self.aot {
            aot.poll_recompilations();
        }
        let mut budget_consumed = 0;
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

            if budget_consumed == count {
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
                if let Some(handle) = aot.entries.get(&key).copied() {
                    let mut frame = RunFrame {
                        regs: core::array::from_fn(|index| self.cpu.reg_get(Mode::User, index as u8)),
                        cpsr,
                        end,
                        ..RunFrame::default()
                    };
                    let result = {
                        let mut access = MemoryAccess {
                            memory: &mut self.mem,
                            resolve: &aot.entries,
                        };
                        aot.executor.execute(handle, &mut frame, &mut access)
                    };
                    let exit = match result {
                        Ok(exit) => exit,
                        Err(error) => {
                            tracing::warn!(%error, "ARM AOT execution failed; using interpreter");
                            self.aot = None;
                            CompiledExit::Dispatch
                        }
                    };
                    for (index, value) in frame.regs.into_iter().enumerate() {
                        self.cpu.reg_set(Mode::User, index as u8, value);
                    }
                    self.cpu.reg_set(Mode::User, reg::CPSR, frame.cpsr);
                    if exit == CompiledExit::GuestFault {
                        return Err(WieError::InvalidMemoryAccess(frame.fault_address));
                    }
                    interpret_one = exit == CompiledExit::InterpretOne;
                    lookup_entry = true;
                    continue;
                }
                aot.recompile(&key, &self.mem);
            }
            let recheck_after_step = interpret_one;
            interpret_one = false;
            lookup_entry = false;

            let cache_invalidation = self.instruction_cache_invalidation(pc, cpsr);
            let mut arm32cpu_memory = self.mem.as_arm32cpu_memory();
            if !(self.cpu.step(&mut arm32cpu_memory)) {
                return Err(WieError::FatalError("Undefined instruction".into()));
            }
            if let Some(x) = arm32cpu_memory.memory_error {
                return Err(WieError::InvalidMemoryAccess(x));
            }
            budget_consumed += 1;
            let next_pc = self.cpu.reg_get(Mode::User, reg::PC);
            let next_cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            if next_pc != pc.wrapping_add(if cpsr & 0x20 != 0 { 2 } else { 4 }) || (next_cpsr ^ cpsr) & 0x0100_003f != 0 {
                lookup_entry = true;
            }
            if let Some(invalidation) = cache_invalidation
                && let Some(aot) = &mut self.aot
            {
                let range = match invalidation {
                    InstructionCacheInvalidation::All => 0..TOTAL_MEMORY,
                    InstructionCacheInvalidation::Address(address) => {
                        // ARM926EJ-S cache lines contain 32 bytes.
                        let start = u64::from(address & !31);
                        start..start + 32
                    }
                };
                aot.invalidate(range);
                lookup_entry = true;
            }
            lookup_entry |= recheck_after_step;
        };

        Ok(EngineRunResult {
            stop_reason,
            budget_consumed,
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

    fn record_image(&mut self, address: u32, size: usize) {
        if let Some(aot) = &mut self.aot {
            aot.record_image(address, size);
        }
    }

    fn begin_preparation(&mut self) -> Result<Option<PreparationFuture>> {
        if let Some(aot) = &mut self.aot {
            return aot.begin(&self.mem);
        }
        Ok(None)
    }

    fn is_preparing(&self) -> bool {
        self.aot.as_ref().is_some_and(|aot| aot.state != PreparationState::Ready)
    }

    fn finish_preparation(&mut self, result: Result<CompiledArtifact>) {
        if let Some(aot) = &mut self.aot
            && !aot.finish(result)
        {
            self.aot = None;
        }
    }
}

struct MemoryAccess<'a> {
    memory: &'a mut EmulatedMemory,
    resolve: &'a HashMap<RegionKey, CompiledHandle>,
}

impl ExecutionAccess for MemoryAccess<'_> {
    fn pages(&mut self) -> &mut [MemoryPage; 0x10000] {
        &mut self.memory.pages
    }

    fn resolve(&self, pc: u32, cpsr: u32) -> Option<CompiledHandle> {
        if cpsr & 0x0100_0000 != 0 {
            return None;
        }
        self.resolve
            .get(&RegionKey {
                pc,
                thumb: cpsr & 0x20 != 0,
                cpu_mode: (cpsr & 0x1f) as u8,
            })
            .copied()
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
            let page = self.memory.pages[start].bytes.as_mut()?;
            return Some((&mut page[offset..offset + len], &mut []));
        }
        let end = address.wrapping_add((words - 1) * 4) as usize / PAGE_SIZE;
        let (first, second) = if start < end {
            let (lower, upper) = self.memory.pages.split_at_mut(end);
            (lower[start].bytes.as_mut()?, upper[0].bytes.as_mut()?)
        } else {
            let (lower, upper) = self.memory.pages.split_at_mut(start);
            (upper[0].bytes.as_mut()?, lower[end].bytes.as_mut()?)
        };
        Some((&mut first[offset..], &mut second[..len - first_len]))
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

pub(crate) struct EmulatedMemory {
    pages: Box<[MemoryPage; 0x10000]>,
}

impl EmulatedMemory {
    fn new() -> Self {
        Self {
            pages: (0..TOTAL_MEMORY / PAGE_SIZE as u64)
                .map(|_| MemoryPage::default())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .ok()
                .unwrap(),
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
            if page_data.bytes.is_none() {
                page_data.bytes = Some(Box::new([0; PAGE_SIZE]));
            }
        }
    }

    fn read_range(&self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        let mut remaining_size = size;
        let mut current_address = address;

        while remaining_size > 0 {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE]
                .bytes
                .as_ref()
                .ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(remaining_size);

            result[size - remaining_size..size - remaining_size + available_bytes].copy_from_slice(&page_data[offset..offset + available_bytes]);
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
            let page_data = &mut self.pages[page_address as usize / PAGE_SIZE];
            let bytes = page_data.bytes.as_mut().ok_or(WieError::InvalidMemoryAccess(current_address))?;
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(data.len() - data_index);

            bytes[offset..offset + available_bytes].copy_from_slice(&data[data_index..data_index + available_bytes]);
            data_index += available_bytes;
            current_address += available_bytes as u32;
        }

        Ok(())
    }

    pub(crate) fn code_image(&self, address: u32, size: usize) -> Result<CodeImage> {
        let mut bytes = alloc::vec![0; size];
        self.read_range(address, size, &mut bytes)?;
        Ok(CodeImage { address, bytes })
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        if self.pages[page_start as usize / PAGE_SIZE].bytes.is_none() {
            return false;
        }

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            if self.pages[page as usize / PAGE_SIZE].bytes.is_none() {
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

    fn get_page(&mut self, addr: u32) -> Option<&mut [u8; PAGE_SIZE]> {
        let page_address = addr & !PAGE_MASK;
        let page_data = self.emulated_memory.pages[page_address as usize / PAGE_SIZE].bytes.as_deref_mut();

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

        data[offset as usize]
    }

    fn r16(&mut self, addr: u32) -> u16 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();

        u16::from_le_bytes(data[offset as usize..offset as usize + 2].try_into().unwrap())
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap();
        u32::from_le_bytes(data[offset as usize..offset as usize + 4].try_into().unwrap())
    }

    fn w8(&mut self, addr: u32, val: u8) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize] = val;
    }

    fn w16(&mut self, addr: u32, val: u16) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize..offset as usize + 2].copy_from_slice(&val.to_le_bytes());
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let data = page.unwrap();

        data[offset as usize..offset as usize + 4].copy_from_slice(&val.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec};
    use core::{
        sync::atomic::{AtomicBool, Ordering},
        task::Poll,
    };
    use std::{
        thread,
        time::{Duration, Instant},
    };

    use arm32_cpu::Memory;
    use spin::Mutex;
    use test_utils::TestNativeExecutor;
    use wie_arm_jit_types::{CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion};

    use crate::engine::{ArmEngine, ArmRegister, EngineStopReason, MemoryPermission};

    use super::*;

    #[test]
    fn memory_reads_wrap_at_the_last_guest_page() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem.pages[0xffff] = MemoryPage {
            bytes: Some(Box::new([b'x'; PAGE_SIZE])),
        };
        engine.mem_map(0, PAGE_SIZE, MemoryPermission::ReadWrite);
        engine.mem_write(0, b"y").unwrap();
        let mut bytes = [0; 2];
        assert_eq!(engine.mem_read(u32::MAX, 2, &mut bytes).unwrap(), 2);
        assert_eq!(&bytes, b"xy");
    }

    #[derive(Default)]
    struct Responses {
        requests: Vec<Vec<wie_arm_jit_types::CompileRegion>>,
        ready: Option<futures::channel::oneshot::Sender<Result<CompiledArtifact>>>,
        released: Vec<CompiledHandle>,
        executed: Vec<u32>,
    }

    struct DeferredExecutor(Arc<Mutex<Responses>>);

    impl CompiledExecutor for DeferredExecutor {
        fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
            let (sender, receiver) = futures::channel::oneshot::channel();
            let mut state = self.0.lock();
            state.requests.push(request.regions.flatten().collect());
            state.ready = Some(sender);
            Box::pin(async move {
                receiver
                    .await
                    .unwrap_or_else(|_| Err(WieError::FatalError("preparation cancelled".into())))
            })
        }

        fn release(&mut self, handle: CompiledHandle) {
            self.0.lock().released.push(handle);
        }

        fn execute(&mut self, handle: CompiledHandle, _: &mut RunFrame, _: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
            self.0.lock().executed.push(handle.module);
            Ok(CompiledExit::InterpretOne)
        }
    }

    fn artifact(request: &[wie_arm_jit_types::CompileRegion], module: u32) -> CompiledArtifact {
        CompiledArtifact {
            regions: request
                .iter()
                .filter(|region| region.ir.entry.thumb)
                .enumerate()
                .map(|(slot, region)| CompiledRegion {
                    manifest: ManifestRegion {
                        entry: RegionKey { ..region.ir.entry },
                        instruction_pcs: region
                            .ir
                            .blocks
                            .iter()
                            .flat_map(|block| &block.instructions)
                            .take(2)
                            .map(|instruction| instruction.pc.get())
                            .collect(),
                        code_ranges: region
                            .ir
                            .blocks
                            .iter()
                            .map(|block| {
                                let last = block.instructions.last().unwrap();
                                u64::from(block.instructions[0].pc.get())..u64::from(last.pc.get()) + u64::from(last.size)
                            })
                            .collect(),
                    },
                    handle: CompiledHandle { module, slot: slot as u32 },
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
        let _preparation = aot.begin(&memory).unwrap().unwrap();

        let state = responses.lock();
        assert_eq!(state.requests.len(), 1);
        let request = &state.requests[0];
        let instructions: Vec<_> = request
            .iter()
            .filter(|region| region.ir.entry.thumb)
            .flat_map(|region| &region.ir.blocks)
            .flat_map(|block| &block.instructions)
            .collect();
        assert_eq!(
            instructions.iter().map(|instruction| instruction.pc.get()).collect::<Vec<_>>(),
            [0x3ffe, 0x4000, 0x4002]
        );
        for (instruction, immediate) in instructions.iter().zip([1, 7]) {
            assert!(
                matches!(instruction.operation, wie_arm_jit_types::ir::Operation::Alu { right: wie_arm_jit_types::ir::Operand { value: wie_arm_jit_types::ir::Value::Immediate(value), .. }, .. } if value == immediate)
            );
        }
        assert!(aot.state == PreparationState::Preparing);
        drop(state);
        assert!(aot.begin(&memory).unwrap().is_none());
        assert_eq!(responses.lock().requests.len(), 1);
        assert!(aot.state == PreparationState::Preparing);
    }

    #[test]
    fn invalidated_regions_recompile_on_entry_without_blocking_other_code() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut engine = Arm32CpuEngine::with_backend(Some(Box::new(DeferredExecutor(responses.clone()))));
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        for pc in [0x1000, 0x1040] {
            engine.mem_write(pc, &[1, 0x30, 0x70, 0x47]).unwrap();
            engine.record_image(pc, 4);
        }
        let preparation = engine.begin_preparation().unwrap().unwrap();
        let compiled = artifact(&responses.lock().requests[0], 0);
        assert!(responses.lock().ready.take().unwrap().send(Ok(compiled)).is_ok());
        engine.finish_preparation(futures::executor::block_on(preparation));
        let key = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        for wrong in [
            RegionKey { pc: 0x1001, ..key },
            RegionKey { pc: 0x1004, ..key },
            RegionKey { thumb: false, ..key },
            RegionKey { cpu_mode: 0x13, ..key },
        ] {
            assert!(!engine.aot.as_ref().unwrap().entries.contains_key(&wrong));
        }
        engine.mem_write(0x1000, &[2, 0x30]).unwrap();
        engine.mem.as_arm32cpu_memory().w16(0x1000, 0x3003);
        engine.aot.as_mut().unwrap().invalidate(0x1020..0x1040);
        assert!(engine.aot.as_ref().unwrap().entries.contains_key(&key));

        // MVA addresses any byte in the cache line, not only the region's entry PC.
        engine.mem_write(0x1080, &0xee070f35u32.to_le_bytes()).unwrap();
        engine.reg_write(ArmRegister::Cpsr, 0x1f);
        engine.reg_write(ArmRegister::PC, 0x1080);
        engine.reg_write(ArmRegister::R0, 0x101f);
        engine.run(0x1084, 1).unwrap();
        let aot = engine.aot.as_ref().unwrap();
        assert!(!aot.entries.contains_key(&key));
        assert!(!aot.entries.contains_key(&RegionKey { pc: 0x1002, ..key }));
        assert!(aot.entries.contains_key(&RegionKey { pc: 0x1040, ..key }));
        assert_eq!(responses.lock().requests.len(), 1);

        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2000);
        engine.reg_write(ArmRegister::R0, 0);
        engine.run(0x2000, 10).unwrap();
        assert_eq!(engine.reg_read(ArmRegister::R0), 3);
        assert!(responses.lock().executed.is_empty());
        assert_eq!(responses.lock().requests.len(), 2);
        assert!(responses.lock().requests[1].iter().all(|region| region.ir.entry.thumb));
        engine.reg_write(ArmRegister::PC, 0x1041);
        engine.run(0x2000, 10).unwrap();
        assert_eq!(responses.lock().executed.as_slice(), &[0, 0]);

        // A second explicit flush supersedes a pending snapshot; no page versions are needed.
        engine.mem.as_arm32cpu_memory().w16(0x1000, 0x3004);
        engine.aot.as_mut().unwrap().invalidate(0x1000..0x1020);
        let stale = artifact(&responses.lock().requests[1], 1);
        assert!(responses.lock().ready.take().unwrap().send(Ok(stale)).is_ok());
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::R0, 0);
        engine.run(0x2000, 10).unwrap();
        assert_eq!(engine.reg_read(ArmRegister::R0), 4);
        assert_eq!(responses.lock().requests.len(), 3);
        assert!(!engine.aot.as_ref().unwrap().entries.contains_key(&key));
        assert_eq!(responses.lock().released.iter().map(|handle| handle.module).collect::<Vec<_>>(), [0, 1]);

        let fresh = artifact(&responses.lock().requests[2], 2);
        assert!(responses.lock().ready.take().unwrap().send(Ok(fresh)).is_ok());
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.run(0x2000, 10).unwrap();
        assert_eq!(engine.aot.as_ref().unwrap().entries.get(&key).unwrap().module, 2);
        assert_eq!(responses.lock().executed.as_slice(), &[0, 0, 2, 2]);

        engine.aot.as_mut().unwrap().invalidate(0..TOTAL_MEMORY);
        assert!(engine.aot.as_ref().unwrap().entries.is_empty());
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.run(0x2000, 10).unwrap();
        assert!(
            responses
                .lock()
                .ready
                .take()
                .unwrap()
                .send(Err(WieError::FatalError("compile failed".into())))
                .is_ok()
        );
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.run(0x2000, 10).unwrap();
        assert!(!engine.aot.as_ref().unwrap().entries.contains_key(&key));
        assert_eq!(responses.lock().requests.len(), 4);
    }

    #[test]
    fn failed_preparation_does_not_install_late_results() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut aot = Aot::new(Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        memory.map(0x1000, 0x1000);
        aot.record_image(0x1000, 4);
        let preparation = aot.begin(&memory).unwrap().unwrap();
        assert!(
            responses
                .lock()
                .ready
                .take()
                .unwrap()
                .send(Err(WieError::FatalError("compile failed".into())))
                .is_ok()
        );
        assert!(!aot.finish(futures::executor::block_on(preparation)));
        assert!(aot.state == PreparationState::Ready);
        let late = artifact(&responses.lock().requests[0], 0);
        assert!(!aot.finish(Ok(late)));
        assert!(aot.entries.is_empty());
        assert_eq!(responses.lock().requests.len(), 1);
    }

    struct TestExecutor {
        calls: Arc<Mutex<(u32, bool)>>,
        completed: Option<u32>,
    }

    impl CompiledExecutor for TestExecutor {
        fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
            let compiled = artifact(&request.regions.flatten().collect::<Vec<_>>(), 0);
            Box::pin(async move { Ok(compiled) })
        }

        fn release(&mut self, _: CompiledHandle) {}

        fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
            self.calls.lock().0 += 1;
            assert_eq!(access.resolve(frame.regs[15], frame.cpsr).map(|handle| handle.slot), Some(handle.slot));
            assert!(access.resolve(frame.regs[15], frame.cpsr | 0x0100_0000).is_none());
            if let Some(completed) = self.completed {
                if completed != 0 {
                    access.pages()[2].bytes.as_mut().unwrap()[..4].copy_from_slice(&42u32.to_le_bytes());
                    frame.regs[0] = 43;
                    frame.regs[15] += completed * 2;
                }
                return Err(WieError::FatalError("injected backend failure at an instruction boundary".into()));
            }
            Ok(CompiledExit::InterpretOne)
        }
    }

    impl Drop for TestExecutor {
        fn drop(&mut self) {
            self.calls.lock().1 = true;
        }
    }

    #[test]
    fn interpreter_handoff_and_backend_failure_preserve_progress_and_budget() {
        for (completed, budget) in [(None, 10), (Some(0), 10), (Some(2), 10), (Some(2), 2)] {
            let calls = Arc::new(Mutex::new((0, false)));
            let mut engine = Arm32CpuEngine::with_backend(Some(Box::new(TestExecutor {
                calls: calls.clone(),
                completed,
            })));
            engine.mem_map(0x1000, 6, MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 4, MemoryPermission::ReadWrite);
            engine.mem_write(0x1000, &[0x08, 0x60, 0x01, 0x30, 0x70, 0x47]).unwrap(); // str r0, [r1]; add r0, #1; bx lr
            engine.reg_write(ArmRegister::R0, 42);
            engine.reg_write(ArmRegister::R1, 0x20000);
            engine.reg_write(ArmRegister::Cpsr, 0x3f);
            engine.reg_write(ArmRegister::PC, 0x1001);
            engine.reg_write(ArmRegister::LR, 0x2000);
            engine.record_image(0x1000, 6);
            let preparation = engine.begin_preparation().unwrap().unwrap();
            engine.finish_preparation(futures::executor::block_on(preparation));
            let result = engine.run(0x2000, budget).unwrap();
            assert_eq!(result.budget_consumed, 3 - completed.unwrap_or(0));
            assert_eq!(engine.reg_read(ArmRegister::Cpsr), 0x1f);
            assert_eq!(engine.aot.is_some(), completed.is_none());
            assert_eq!(calls.lock().0, if completed.is_some() { 1 } else { 2 });
            assert_eq!(engine.reg_read(ArmRegister::R0), 43);
            let mut value = [0; 4];
            engine.mem_read(0x20000, 4, &mut value).unwrap();
            assert_eq!(u32::from_le_bytes(value), 42);
            assert_eq!(calls.lock().1, completed.is_some());
            assert!(matches!(result.stop_reason, EngineStopReason::End));
        }
    }

    #[test]
    fn preparation_waits_without_running_image_code() {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut core = crate::ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: Some(Box::new(DeferredExecutor(responses.clone()))),
            profile: None,
        })
        .unwrap();
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
        assert!(
            responses
                .lock()
                .ready
                .take()
                .unwrap()
                .send(Ok(CompiledArtifact { regions: Vec::new() }))
                .is_ok()
        );
        assert!(controller.is_preparing());
        assert!(matches!(execution.as_mut().poll(&mut context), Poll::Ready(Ok(1))));
        assert!(!controller.is_preparing());
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
                completed: None,
            }));
            aot.record_image(0x1000, 4);
            let preparation = aot.begin(&engine.mem).unwrap().unwrap();
            aot.finish(futures::executor::block_on(preparation));
            engine.aot = Some(aot);
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, 0x1001);
            assert_eq!(engine.run(0x1002, 1).unwrap().budget_consumed, 1);
            assert_eq!(engine.reg_read(ArmRegister::R0), 1);
            assert_eq!(calls.lock().0, u32::from(cpsr == 0x3f));
        }
    }

    #[test]
    fn word_range_admission_checks_mapping_without_reading() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, PAGE_SIZE);
        memory.pages[0xffff] = MemoryPage {
            bytes: Some(Box::new([0; PAGE_SIZE])),
        };
        memory.write_range(0x10000, &42u32.to_le_bytes()).unwrap();
        let mut access = MemoryAccess {
            memory: &mut memory,
            resolve: &HashMap::new(),
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
        assert_eq!(access.memory.as_arm32cpu_memory().r32(0x10000), 42);
        access.memory.map(0x20000, PAGE_SIZE);
        access.memory.map(0, PAGE_SIZE);
        assert!(access.word_range(0xffff_fffd, 1).is_none());
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
    }

    #[test]
    fn coprocessor_cache_maintenance_targets_regions_in_the_active_register_bank() {
        for (opcode, mode, cpsr, register, operand, invalidated) in [
            (0xee070f15u32, Mode::System, 0x1f, 0, 0, true),
            (0x0e070f15, Mode::System, 0x1f, 0, 0, true),
            (0xee070f17, Mode::System, 0x1f, 0, 0, true),
            (0xee070f55, Mode::System, 0x1f, 0, 0, true),
            (0xee070f35, Mode::System, 0x1f, 0, 0x20020, true),
            (0xee070f35, Mode::System, 0x1f, 0, 0x20040, false),
            (0xee070f35, Mode::Supervisor, 0x13, 13, 0x20020, true),
            (0xee070f35, Mode::Fiq, 0x11, 8, 0x20020, true),
            (0xee070e15, Mode::System, 0x1f, 0, 0, false),
            (0xee270f15, Mode::System, 0x1f, 0, 0, false),
            (0xee170f15, Mode::System, 0x1f, 0, 0, false),
            (0xee070f16, Mode::System, 0x1f, 0, 0, false),
            (0xee070f95, Mode::System, 0x1f, 0, 0, false),
        ] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut engine = Arm32CpuEngine::with_backend(Some(Box::new(DeferredExecutor(responses.clone()))));
            engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 0x10000, MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, &(opcode | u32::from(register) << 12).to_le_bytes()).unwrap();
            // A four-byte Thumb BL crosses the invalidated line at 0x20020.
            engine.mem_write(0x2001e, &[0, 0xf0, 0, 0xf8, 0x70, 0x47]).unwrap();
            engine.record_image(0x2001e, 6);
            let preparation = engine.begin_preparation().unwrap().unwrap();
            let compiled = artifact(&responses.lock().requests[0], 0);
            assert!(responses.lock().ready.take().unwrap().send(Ok(compiled)).is_ok());
            engine.finish_preparation(futures::executor::block_on(preparation));
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.cpu.reg_set(Mode::User, register, 0x30000);
            engine.cpu.reg_set(mode, register, operand);
            let result = engine.run(0x1004, 1).unwrap();
            assert_eq!(result.budget_consumed, 1);
            assert!(matches!(result.stop_reason, EngineStopReason::End));
            assert_eq!(
                engine.aot.as_ref().unwrap().entries.is_empty(),
                invalidated,
                "opcode={opcode:#x}, mode={mode:?}"
            );
            assert_eq!(engine.reg_read(ArmRegister::Cpsr), cpsr);
        }
    }

    #[test]
    fn run_reports_consumed_budget_at_yield_and_return_boundaries() {
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0xc0, 0x46, 0xc0, 0x46, 0x70, 0x47]).unwrap(); // nop; nop; bx lr
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.reg_write(ArmRegister::LR, 0x2000);

        for (budget, expected_count, at_end) in [(0, 0, false), (2, 2, false), (10, 1, true), (10, 0, true)] {
            let result = engine.run(0x2000, budget).unwrap();
            assert_eq!(result.budget_consumed, expected_count);
            assert!(matches!(
                (result.stop_reason, at_end),
                (EngineStopReason::End, true) | (EngineStopReason::Yield, false)
            ));
        }
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

    struct Engines {
        interpreted: Arm32CpuEngine,
        native: Arm32CpuEngine,
        executed: Arc<AtomicBool>,
    }

    impl Engines {
        fn new(bytes: &[u8]) -> Self {
            let executor = TestNativeExecutor::new();
            let executed = executor.executed.clone();
            let mut engines = Self {
                interpreted: Arm32CpuEngine::new(),
                native: Arm32CpuEngine::with_backend(Some(Box::new(executor))),
                executed,
            };
            for engine in [&mut engines.interpreted, &mut engines.native] {
                engine.mem_map(0x1000, bytes.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, bytes).unwrap();
                engine.mem_map(0x20000, 0x20000, MemoryPermission::ReadWrite);
            }
            engines.native.record_image(0x1000, bytes.len());
            let preparation = engines.native.begin_preparation().unwrap().unwrap();
            let artifact = futures::executor::block_on(preparation).unwrap();
            engines.native.finish_preparation(Ok(artifact));
            engines
        }

        fn reset(&mut self, pc: u32, cpsr: u32, registers: &[(ArmRegister, u32)]) {
            for engine in [&mut self.interpreted, &mut self.native] {
                for index in 0..15 {
                    engine.cpu.reg_set(arm32_cpu::Mode::User, index, 0);
                }
                engine.reg_write(ArmRegister::Cpsr, cpsr);
                engine.reg_write(ArmRegister::PC, pc);
                engine.reg_write(ArmRegister::LR, 0x8000);
                for &(register, value) in registers {
                    engine.reg_write(register, value);
                }
            }
            self.executed.store(false, Ordering::Relaxed);
        }

        fn compare_run(&mut self, end: u32) -> bool {
            let pc = self.native.reg_read(ArmRegister::PC);
            let expected = self.interpreted.run(end, u32::MAX);
            let actual = self.native.run(end, u32::MAX);
            match (actual, expected) {
                (Ok(actual), Ok(expected)) => {
                    for index in 0..=16 {
                        assert_eq!(
                            self.native.cpu.reg_get(arm32_cpu::Mode::User, index),
                            self.interpreted.cpu.reg_get(arm32_cpu::Mode::User, index),
                            "register {index}, pc={pc:#x}, end={end:#x}"
                        );
                    }
                    if expected.budget_consumed > 0 {
                        assert!(self.executed.load(Ordering::Relaxed), "native code must execute at {pc:#x}");
                    }
                    match (&actual.stop_reason, expected.stop_reason) {
                        (EngineStopReason::End, EngineStopReason::End) | (EngineStopReason::Yield, EngineStopReason::Yield) => {}
                        (
                            EngineStopReason::Svc { category, lr, spsr },
                            EngineStopReason::Svc {
                                category: ec,
                                lr: el,
                                spsr: es,
                            },
                        ) => {
                            assert_eq!((*category, *lr, *spsr), (ec, el, es));
                        }
                        _ => panic!("different stop reasons"),
                    }
                    true
                }
                (Err(WieError::InvalidMemoryAccess(_)), Err(WieError::InvalidMemoryAccess(_))) => false,
                (Err(actual), Err(expected)) => {
                    assert_eq!(alloc::format!("{actual:?}"), alloc::format!("{expected:?}"));
                    false
                }
                _ => panic!("different execution outcomes"),
            }
        }

        fn compare_memory(&self) {
            for page in [0, 2, 3, 0xffff] {
                assert!(
                    self.native.mem.pages[page].bytes == self.interpreted.mem.pages[page].bytes,
                    "memory page {page:#x}"
                );
            }
        }
    }

    #[test]
    fn native_memory_transfers_preserve_completed_prefix() {
        let mut opcodes = vec![
            0xe581_2000,
            0xe591_2000,
            0xe5c1_2000,
            0xe5d1_2000,
            0xe1c1_20b0,
            0xe1d1_20b0,
            0xe1d1_20d0,
            0xe1d1_20f0,
            0xe101_2093,
            0xe141_2093,
            0xe101_2092,
            0xe581_f000,
            0xe5a1_2004,
            0xe481_2004,
            0xe1c2_20d0,
            0x0591_2000,
        ];
        for load in [0, 1] {
            for increment in [0, 1] {
                for before in [0, 1] {
                    for write_back in [0, 1] {
                        opcodes.push(0xe801_003c | load << 20 | write_back << 21 | increment << 23 | before << 24);
                    }
                }
            }
            for pre_index in [0, 1] {
                for subtract in [0, 1] {
                    opcodes.push(0xe041_2094 | (if load == 1 { 2 } else { 3 }) << 5 | pre_index << 24 | (1 - subtract) << 23);
                    opcodes.push(0xe401_2004 | load << 20 | pre_index << 24 | (1 - subtract) << 23);
                }
            }
        }
        opcodes.extend([0xe8a1_0006, 0xe891_0006]);
        // str r6,[r7] precedes every operation so a memory fault must preserve earlier writes.
        let bytes: Vec<_> = opcodes
            .iter()
            .flat_map(|opcode| [0xe587_6000, *opcode, 0xe12f_ff1e])
            .flat_map(u32::to_le_bytes)
            .collect();
        let mut engines = Engines::new(&bytes);
        for (index, opcode) in opcodes.into_iter().enumerate() {
            let pc = 0x1000 + index as u32 * 12;
            assert!(
                engines.native.aot.as_ref().unwrap().entries.contains_key(&RegionKey {
                    pc: pc + 4,
                    thumb: false,
                    cpu_mode: 0x1f
                }),
                "{opcode:08x}"
            );
            for mapped_second in [false, true] {
                for address in [0x21000u32, 0x21004, 0x2fff8, 0x2fffc, 0x30000, 0x40000, 0xffff_fff8, 0xffff_fffc] {
                    if opcode & 0x0e10_00d0 == 0x0000_00d0 {
                        let offset = ((opcode >> 4) & 0xf0) | (opcode & 15);
                        let effective = if opcode & 0x0100_0000 == 0 {
                            address
                        } else if opcode & 0x0080_0000 != 0 {
                            address.wrapping_add(offset)
                        } else {
                            address.wrapping_sub(offset)
                        };
                        if effective & 7 != 0 {
                            continue;
                        }
                    }
                    for engine in [&mut engines.interpreted, &mut engines.native] {
                        engine.mem.pages[2].bytes.as_mut().unwrap().fill(0x55);
                        engine.mem.pages[3].bytes = mapped_second.then(|| Box::new([0x66; 0x10000]));
                        engine.mem.pages[0xffff].bytes = Some(Box::new([0x77; 0x10000]));
                    }
                    engines.reset(
                        pc,
                        0x9800_001f,
                        &[
                            (ArmRegister::R1, address),
                            (ArmRegister::R2, if opcode == 0xe1c2_20d0 { address } else { 0x89ab_cdef }),
                            (ArmRegister::R3, 0x7654_3210),
                            (ArmRegister::R4, 0x1122_3344),
                            (ArmRegister::R5, 0x5566_7788),
                            (ArmRegister::R6, 42),
                            (ArmRegister::R7, 0x23000),
                        ],
                    );
                    if engines.compare_run(0x8000) {
                        engines.compare_memory();
                    } else {
                        assert_eq!(engines.native.reg_read(ArmRegister::PC), pc + 4, "{opcode:08x}");
                        assert_eq!(&engines.native.mem.pages[2].bytes.as_ref().unwrap()[0x3000..0x3004], &42u32.to_le_bytes());
                    }
                }
            }
        }
    }

    #[test]
    fn native_branch_exchange_pc_loads_and_interpreter_handoffs_match() {
        let opcodes = [
            0xe1a0_f000u32, // mov pc,r0
            0xe12f_ff10,    // bx r0
            0xe12f_ff30,    // blx r0
            0xe591_f000,    // ldr pc,[r1]
            0xe491_f004,    // ldr pc,[r1],#4
            0xe8b1_8001,    // ldmia r1!,{r0,pc}
            0x0591_f000,    // ldreq pc,[r1]
        ];
        let bytes: Vec<_> = opcodes
            .into_iter()
            .flat_map(|opcode| [opcode, 0xe12f_ff1e])
            .flat_map(u32::to_le_bytes)
            .collect();
        let mut engines = Engines::new(&bytes);
        for index in 0..opcodes.len() {
            for target in [0x8000, 0x8001, 0x8002, 0x8003, 0xffc] {
                // ARMv5 declares unaligned ARM-state PC writes UNPREDICTABLE.
                if target & 3 == 2 || (index == 0 && target & 3 != 0) {
                    continue;
                }
                let pc = 0x1000 + index as u32 * 8;
                engines.reset(pc, 0x1f, &[(ArmRegister::R0, target), (ArmRegister::R1, 0x21000)]);
                for engine in [&mut engines.interpreted, &mut engines.native] {
                    engine.mem_write(0x21000, &target.to_le_bytes().repeat(2)).unwrap();
                }
                engines.compare_run(0x8000);
            }
        }
        // Thumb: str r0,[r1]; svc #4. The SVC remains interpreter-owned.
        let mut engines = Engines::new(&[0x08, 0x60, 0x04, 0xdf]);
        engines.reset(0x1001, 0x3f, &[(ArmRegister::R0, 42), (ArmRegister::R1, 0x21000)]);
        engines.compare_run(0x8000);
        engines.compare_memory();
    }

    #[test]
    fn native_code_changes_only_after_explicit_instruction_cache_invalidation() {
        let mut engines = Engines::new(&[1, 0x20, 0x70, 0x47, 0x08, 0x68, 0x70, 0x47]); // movs r0,#1; bx lr; ldr r0,[r1]; bx lr
        for opcode in [0xee07_0f35u32, 0xee07_0f15] {
            engines.native.mem_write(0x1000, &[1, 0x20]).unwrap();
            engines.reset(0x1001, 0x3f, &[]);
            assert_eq!(engines.native.run(0x8000, 2).unwrap().budget_consumed, 0);
            assert_eq!(engines.native.reg_read(ArmRegister::R0), 1);

            engines.native.mem_write(0x1000, &[2, 0x20]).unwrap();
            engines.reset(0x1001, 0x3f, &[]);
            engines.native.run(0x8000, 2).unwrap();
            assert_eq!(engines.native.reg_read(ArmRegister::R0), 1);

            engines.native.mem_write(0x1800, &opcode.to_le_bytes()).unwrap();
            engines.reset(0x1800, 0x1f, &[(ArmRegister::R0, 0x1000)]);
            engines.native.run(0x1804, 1).unwrap();
            let key = RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            assert!(!engines.native.aot.as_ref().unwrap().entries.contains_key(&key));
            engines.reset(0x1001, 0x3f, &[]);
            assert_eq!(engines.native.run(0x8000, 2).unwrap().budget_consumed, 2);
            assert_eq!(engines.native.reg_read(ArmRegister::R0), 2);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !engines.native.aot.as_ref().unwrap().entries.contains_key(&key) {
                assert!(Instant::now() < deadline, "native recompilation did not finish");
                engines.native.aot.as_mut().unwrap().poll_recompilations();
                thread::yield_now();
            }
            engines.reset(0x1001, 0x3f, &[]);
            assert_eq!(engines.native.run(0x8000, 2).unwrap().budget_consumed, 0);
            assert_eq!(engines.native.reg_read(ArmRegister::R0), 2);

            // Install the original bytes again before testing the next CP15 operation.
            engines.native.mem_write(0x1000, &[1, 0x20]).unwrap();
            engines.native.aot.as_mut().unwrap().invalidate(0x1000..0x1020);
            engines.reset(0x1001, 0x3f, &[]);
            engines.native.run(0x8000, 2).unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !engines.native.aot.as_ref().unwrap().entries.contains_key(&key) {
                assert!(Instant::now() < deadline, "native recompilation did not finish");
                engines.native.aot.as_mut().unwrap().poll_recompilations();
                thread::yield_now();
            }
        }
        // A fault in cached native code must not replay newly written guest bytes.
        engines.native.mem_write(0x1004, &[7, 0x20]).unwrap();
        engines.reset(0x1005, 0x3f, &[(ArmRegister::R1, 0x40000)]);
        assert!(matches!(engines.native.run(0x8000, 2), Err(WieError::InvalidMemoryAccess(0x40000))));
        assert_eq!(engines.native.reg_read(ArmRegister::PC), 0x1004);
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 0);
    }

    #[test]
    fn native_arithmetic_conditions_and_shifts_match_interpreter() {
        let mut opcodes = Vec::new();
        for op in 0..16 {
            let rd = if (8..=11).contains(&op) { 0 } else { 2 };
            opcodes.push(0xe010_0001 | op << 21 | rd << 12);
            if !(8..=11).contains(&op) {
                opcodes.push(0xe000_0001 | op << 21 | rd << 12);
            }
        }
        for kind in 0..4 {
            for set_flags in [0, 1] {
                for immediate in [0, 1, 31] {
                    opcodes.push(0xe1a0_2001 | set_flags << 20 | kind << 5 | immediate << 7);
                }
                opcodes.push(0xe1a0_2311 | set_flags << 20 | kind << 5);
            }
        }
        for set_flags in [0, 1] {
            opcodes.extend([0xe3a0_2000, 0xe3a0_2102, 0xe3e0_2000, 0xe3e0_2102].map(|opcode| opcode | set_flags << 20));
        }
        opcodes.extend([0xe012_0190, 0xe032_4190, 0xe16f_2f11, 0xe10f_2000, 0xe128_f001]);
        for signed in [0, 1] {
            for accumulate in [0, 1] {
                opcodes.push(0xe093_2190 | signed << 22 | accumulate << 21);
            }
        }
        for condition in 0..14 {
            opcodes.push(condition << 28 | 0x0280_2001);
        }
        let chains: &[&[u32]] = &[
            // Full flag writes separated by stores and loads remain dead.
            &[0xe090_2001, 0xe587_2000, 0xe050_3001, 0xe597_5000, 0xe090_4001],
            // adds; movs r3,r0; mrs r4,cpsr; movs r5,r0,lsl #1: preserve C/V, then V.
            &[0xe090_2001, 0xe1b0_3000, 0xe10f_4000, 0xe1b0_5080],
            // cmp; movs r2,r0,lsl r1; adc; sbcs; movs r6,r0,rrx: live carry inputs.
            &[0xe150_0001, 0xe1b0_2110, 0xe0a0_4001, 0xe0d0_5001, 0xe1b0_6060],
            // movs r2,r0,lsl #1; subsvs r3,r0,r1; adc: a skipped write preserves NZC.
            &[0xe1b0_2080, 0x6050_3001, 0xe0a0_4001],
        ];
        let mut bytes: Vec<_> = opcodes
            .iter()
            .flat_map(|opcode| [*opcode, 0xe12f_ff1e])
            .flat_map(u32::to_le_bytes)
            .collect();
        let chain_pc = 0x1000 + bytes.len() as u32;
        bytes.extend(
            chains
                .iter()
                .flat_map(|chain| chain.iter().copied().chain([0xe12f_ff1e]))
                .flat_map(u32::to_le_bytes),
        );
        let mut engines = Engines::new(&bytes);
        for (index, opcode) in opcodes.into_iter().enumerate() {
            let pc = 0x1000 + index as u32 * 8;
            assert!(
                engines.native.aot.as_ref().unwrap().entries.contains_key(&RegionKey {
                    pc,
                    thumb: false,
                    cpu_mode: 0x1f
                }),
                "{opcode:08x}"
            );
            for left in [0, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff] {
                for right in [0, 1, 0x8000_0001, 0xffff_ffff] {
                    for flags in (0..16).map(|flags| flags << 28) {
                        for amount in [0, 1, 31, 32, 33, 255, 256] {
                            if opcode & 0x0fef_0f10 != 0x01a0_0310 && amount != 0 {
                                continue;
                            }
                            engines.reset(
                                pc,
                                flags | 0x0800_001f,
                                &[
                                    (ArmRegister::R0, left),
                                    (ArmRegister::R1, right),
                                    (ArmRegister::R2, 0x7654_3210),
                                    (ArmRegister::R3, amount),
                                    (ArmRegister::R4, 0xffff_ffff),
                                ],
                            );
                            engines.compare_run(0x8000);
                        }
                    }
                }
            }
        }
        let mut pc = chain_pc;
        for chain in chains {
            for start in 0..chain.len() {
                let entry = pc + start as u32 * 4;
                assert!(engines.native.aot.as_ref().unwrap().entries.contains_key(&RegionKey {
                    pc: entry,
                    thumb: false,
                    cpu_mode: 0x1f
                }));
                for left in [0, 1, 0x7fff_ffff, 0x8000_0000, u32::MAX] {
                    for right in [0, 1, 31, 32, 256, 0x8000_0001, u32::MAX] {
                        for flags in (0..16).map(|flags| flags << 28) {
                            engines.reset(
                                entry,
                                flags | 0x0800_001f,
                                &[(ArmRegister::R0, left), (ArmRegister::R1, right), (ArmRegister::R7, 0x21000)],
                            );
                            engines.compare_run(0x8000);
                        }
                    }
                }
            }
            pc += (chain.len() + 1) as u32 * 4;
        }
    }

    #[test]
    fn native_loops_and_interior_entries_run_without_interpreter_budget() {
        // subs r0,#1; bne 0x1000; bx lr
        let bytes: Vec<_> = [0x3801u16, 0xd1fd, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect();
        let mut engines = Engines::new(&bytes);
        for pc in [0x1000, 0x1002, 0x1004] {
            for end in [pc, 0x8000] {
                engines.reset(pc | 1, 0x3f, &[(ArmRegister::R0, 3)]);
                engines.compare_run(end);
            }
        }
        engines.reset(0x1001, 0x3f, &[(ArmRegister::R0, 3)]);
        assert!(matches!(engines.native.run(0x8000, 0).unwrap().stop_reason, EngineStopReason::Yield));
        assert_eq!(engines.native.reg_read(ArmRegister::PC), 0x1000);
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 3);
        let result = engines.native.run(0x8000, 1).unwrap();
        assert!(matches!(result.stop_reason, EngineStopReason::End));
        assert_eq!(result.budget_consumed, 0);
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 0);

        for suffix in [0xf802u16, 0xe802] {
            let mut bytes: Vec<_> = [0xf000, suffix, 0x3001, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect();
            // The callee returns through r4 because BL/BLX replaces LR.
            if suffix == 0xf802 {
                bytes.extend([0x3101u16, 0x4720].into_iter().flat_map(u16::to_le_bytes)); // adds r1,#1; bx r4
            } else {
                bytes.extend([0xe281_1001, 0xe12f_ff14].into_iter().flat_map(u32::to_le_bytes)); // add r1,r1,#1; bx r4
            }
            let mut engines = Engines::new(&bytes);
            engines.reset(0x1001, 0x3f, &[(ArmRegister::R4, 0x8000)]);
            engines.compare_run(0x8000);
            assert_eq!(engines.native.reg_read(ArmRegister::R1), 1);
        }
    }
}
