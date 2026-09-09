use alloc::{boxed::Box, format, vec::Vec};
use core::cell::RefCell;

use arm32_cpu::{Cpu, Memory, Mode, reg};

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
    jit: Option<Jit>,
    closed: bool,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "differential.rs"]
mod differential;

#[cfg(test)]
#[path = "cache_tests.rs"]
mod cache_tests;

impl Arm32CpuEngine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: EmulatedMemory::new(),
            sampler: Sampler::new(),
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
}

impl ArmEngine for Arm32CpuEngine {
    fn run(&mut self, end: u32, count: u32) -> Result<EngineRunResult> {
        if self.closed {
            return Err(WieError::FatalError("ARM core is shut down".into()));
        }
        if let Some(jit) = &mut self.jit {
            jit.poll(&self.mem);
        }
        let mut instructions_executed = 0;
        let mut interpret_one = false;
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
            if !interpret_one
                && cpsr & 0x0100_0000 == 0
                && let Some(jit) = &mut self.jit
            {
                let key = RegionKey {
                    pc,
                    thumb: cpsr & 0x20 != 0,
                    cpu_mode: (cpsr & 0x1f) as u8,
                };
                if let Some((handle, source)) = jit.lookup(key, &self.mem) {
                    let mut frame = RunFrame {
                        regs: core::array::from_fn(|index| self.cpu.reg_get(Mode::User, index as u8)),
                        cpsr,
                        end,
                        budget_remaining: count - instructions_executed,
                        sample_remaining: self.sampler.remaining,
                        ..RunFrame::default()
                    };
                    let mut access = MemoryAccess {
                        memory: &mut self.mem,
                        sampler: &mut self.sampler,
                        source: &source,
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
                    instructions_executed += frame.executed;
                    if let Some((key, hits)) = self.sampler.retire(frame.executed) {
                        jit.sample(key, hits);
                    }
                    if exit == CompiledExit::GuestFault {
                        return Err(WieError::InvalidMemoryAccess(frame.fault_address));
                    }
                    interpret_one = exit == CompiledExit::InterpretOne;
                    continue;
                }
            }
            interpret_one = false;

            if self.sampler.remaining == 1 {
                self.sampler.prepare(pc, cpsr, self.cpu.reg_get(Mode::User, 7), |address, buffer| {
                    self.mem.read_range(address, buffer.len(), buffer).is_ok()
                });
            }

            let mut arm32cpu_memory = self.mem.as_arm32cpu_memory();
            if !(self.cpu.step(&mut arm32cpu_memory)) {
                return Err(WieError::FatalError("Undefined instruction".into()));
            }
            if let Some(x) = arm32cpu_memory.memory_error() {
                return Err(WieError::InvalidMemoryAccess(x));
            }
            instructions_executed += 1;
            if let Some((key, hits)) = self.sampler.retire(1)
                && let Some(jit) = &mut self.jit
            {
                jit.sample(key, hits);
            }
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
    source: &'a [CodePageStamp],
}

impl ExecutionAccess for MemoryAccess<'_> {
    fn load(&mut self, address: u32, width: u32) -> AccessResult {
        if !address.is_multiple_of(width) || self.memory.pages[address as usize / PAGE_SIZE].is_none() {
            return AccessResult::InterpretOne;
        }
        let mut memory = self.memory.as_arm32cpu_memory();
        let value = match width {
            1 => u32::from(memory.r8(address)),
            2 => u32::from(memory.r16(address)),
            _ => memory.r32(address),
        };
        AccessResult::Complete(value)
    }

    fn store(&mut self, address: u32, width: u32, value: u32) -> AccessResult {
        if !address.is_multiple_of(width) || self.memory.pages[address as usize / PAGE_SIZE].is_none() {
            return AccessResult::InterpretOne;
        }
        let mut memory = self.memory.as_arm32cpu_memory();
        match width {
            1 => memory.w8(address, value as u8),
            2 => memory.w16(address, value as u16),
            _ => memory.w32(address, value),
        }
        if self.memory.code_is_current(self.source) {
            AccessResult::Complete(0)
        } else {
            AccessResult::Invalidated
        }
    }

    fn sample_prepare(&mut self, pc: u32, cpsr: u32, r7: u32) {
        self.sampler.prepare(pc, cpsr, r7, |address, buffer| {
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
        data.version = data.version.wrapping_add(1);
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
        data.version = data.version.wrapping_add(1);
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
        data.version = data.version.wrapping_add(1);
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

    struct TestExecutor {
        completion: Option<CompileCompletion>,
        calls: Arc<Mutex<(u32, bool)>>,
        trap: bool,
    }

    impl CompiledExecutor for TestExecutor {
        fn submit(&mut self, request: CompileRequest) -> Admission {
            self.completion = Some(CompileCompletion {
                session: request.session,
                request: request.request,
                result: Ok(CompiledArtifact {
                    regions: request
                        .regions
                        .into_iter()
                        .enumerate()
                        .map(|(slot, region)| CompiledRegion {
                            manifest: ManifestRegion {
                                entry: region.ir.entry,
                                source: region.source,
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
            jit.sample(
                RegionKey {
                    pc: 0x1000,
                    thumb: true,
                    cpu_mode: 0x1f,
                },
                8,
            );
            jit.maintain(&engine.mem);
            engine.jit = Some(jit);
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
    fn code_versions_cover_interpreter_writes_and_partial_host_writes() {
        let mut memory = EmulatedMemory::new();
        memory.map(0x10000, 0x10000);
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        memory.as_arm32cpu_memory().w32(0x10000, 42);
        assert!(!memory.code_is_current(&[before]));
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        assert!(memory.write_range(0x1ffff, &[1, 2]).is_err());
        assert!(!memory.code_is_current(&[before]));
        let (_, _, before) = memory.code_snapshot(0x10000).unwrap();
        let mut sampler = Sampler::new();
        let mut access = MemoryAccess {
            memory: &mut memory,
            sampler: &mut sampler,
            source: &[before],
        };
        assert!(matches!(access.store(0x10001, 4, 42), AccessResult::InterpretOne));
        assert!(access.memory.code_is_current(&[before]));
        assert!(matches!(access.store(0x20000, 4, 42), AccessResult::InterpretOne));
        assert!(matches!(access.store(0x10000, 2, 42), AccessResult::Invalidated));
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
    }

    #[test]
    fn test_memory_unmapped_write() {
        let mut memory = EmulatedMemory::new();

        memory.map(0x10000, 0x10000);

        assert!(memory.write_range(0x1f500, &[12; 0x1000]).is_err());
    }
}
