use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::time::Duration;

use crossbeam::channel;
use spin::Mutex;
use wie_util::WieError;

use crate::{ThreadId, context::ArmCoreContext};

use super::{Arm32CpuEngine, ArmEngine, ArmRegister, EngineRunResult, EngineStopReason, MemoryPermission};

#[derive(Copy, Clone)]
enum ResumeMode {
    Continue,
    Step(ThreadId),
}

#[derive(Copy, Clone)]
enum RunState {
    Initial,
    Paused,
    Running(ResumeMode),
    Interrupt,
}

#[derive(Copy, Clone)]
pub(crate) enum DebugSignal {
    Int,
    Kill,
    Segv,
    Sys,
    Trap,
    Abrt,
}

#[derive(Copy, Clone)]
pub(crate) enum DebugStopReason {
    Signal(DebugSignal, ThreadId),
    SwBreak(ThreadId),
    DoneStep(ThreadId),
}

#[derive(Copy, Clone)]
pub(crate) enum DebugBreakpointKind {
    Thumb16,
    Thumb32,
    Arm32,
}

#[derive(Copy, Clone)]
struct SoftwareBreakpoint {
    kind: DebugBreakpointKind,
    original: [u8; 4],
    patched: bool,
}

fn overlay_breakpoint_originals(breakpoints: &BTreeMap<u32, SoftwareBreakpoint>, start_addr: u32, data: &mut [u8], filter_patched: bool) {
    let end = start_addr + data.len() as u32;

    for (&addr, bp) in breakpoints.iter() {
        if filter_patched && !bp.patched {
            continue;
        }

        let bp_end = addr + bp.kind.original_len() as u32;
        if addr < end && bp_end > start_addr {
            let overlap_start = addr.max(start_addr);
            let overlap_end = bp_end.min(end);

            for offset in overlap_start..overlap_end {
                data[(offset - start_addr) as usize] = bp.original[(offset - addr) as usize];
            }
        }
    }
}

fn update_breakpoint_originals(breakpoints: &mut BTreeMap<u32, SoftwareBreakpoint>, start_addr: u32, data: &[u8]) {
    let end = start_addr + data.len() as u32;

    for (&addr, bp) in breakpoints.iter_mut() {
        let bp_end = addr + bp.kind.original_len() as u32;
        if addr < end && bp_end > start_addr {
            let overlap_start = addr.max(start_addr);
            let overlap_end = bp_end.min(end);

            for offset in overlap_start..overlap_end {
                bp.original[(offset - addr) as usize] = data[(offset - start_addr) as usize];
            }
        }
    }
}

impl DebugBreakpointKind {
    fn original_len(self) -> usize {
        match self {
            Self::Thumb16 => 2,
            Self::Thumb32 => 4,
            Self::Arm32 => 4,
        }
    }

    fn patch_len(self) -> usize {
        match self {
            Self::Thumb16 | Self::Thumb32 => 2,
            Self::Arm32 => 4,
        }
    }

    fn patch_bytes(self) -> [u8; 4] {
        match self {
            Self::Thumb16 | Self::Thumb32 => [0x00, 0xbe, 0x00, 0x00],
            Self::Arm32 => 0xe1200070u32.to_le_bytes(),
        }
    }
}

pub(crate) struct DebugInner {
    cpu: Mutex<Arm32CpuEngine>,
    stop_event_tx: channel::Sender<DebugStopReason>,
    stop_event_rx: channel::Receiver<DebugStopReason>,
    resume_tx: channel::Sender<()>,
    resume_rx: channel::Receiver<()>,
    run_state: Mutex<RunState>,
    resumed_threads: Mutex<Option<Vec<ThreadId>>>,
    breakpoints: Mutex<BTreeMap<u32, SoftwareBreakpoint>>,
    current_thread: Mutex<Option<ThreadId>>,
}

impl DebugInner {
    fn new() -> Arc<Self> {
        let (stop_event_tx, stop_event_rx) = channel::unbounded();
        let (resume_tx, resume_rx) = channel::bounded(1);

        Arc::new(Self {
            cpu: Mutex::new(Arm32CpuEngine::new()),
            stop_event_tx,
            stop_event_rx,
            resume_tx,
            resume_rx,
            run_state: Mutex::new(RunState::Initial),
            resumed_threads: Mutex::new(None),
            breakpoints: Mutex::new(BTreeMap::new()),
            current_thread: Mutex::new(None),
        })
    }

    pub(crate) fn read_registers(&self) -> ArmCoreContext {
        let cpu = self.cpu.lock();

        ArmCoreContext {
            r0: cpu.reg_read(ArmRegister::R0),
            r1: cpu.reg_read(ArmRegister::R1),
            r2: cpu.reg_read(ArmRegister::R2),
            r3: cpu.reg_read(ArmRegister::R3),
            r4: cpu.reg_read(ArmRegister::R4),
            r5: cpu.reg_read(ArmRegister::R5),
            r6: cpu.reg_read(ArmRegister::R6),
            r7: cpu.reg_read(ArmRegister::R7),
            r8: cpu.reg_read(ArmRegister::R8),
            sb: cpu.reg_read(ArmRegister::SB),
            sl: cpu.reg_read(ArmRegister::SL),
            fp: cpu.reg_read(ArmRegister::FP),
            ip: cpu.reg_read(ArmRegister::IP),
            sp: cpu.reg_read(ArmRegister::SP),
            lr: cpu.reg_read(ArmRegister::LR),
            pc: cpu.reg_read(ArmRegister::PC),
            cpsr: cpu.reg_read(ArmRegister::Cpsr),
        }
    }

    pub(crate) fn write_registers(&self, regs: &ArmCoreContext) {
        let mut cpu = self.cpu.lock();

        cpu.reg_write(ArmRegister::R0, regs.r0);
        cpu.reg_write(ArmRegister::R1, regs.r1);
        cpu.reg_write(ArmRegister::R2, regs.r2);
        cpu.reg_write(ArmRegister::R3, regs.r3);
        cpu.reg_write(ArmRegister::R4, regs.r4);
        cpu.reg_write(ArmRegister::R5, regs.r5);
        cpu.reg_write(ArmRegister::R6, regs.r6);
        cpu.reg_write(ArmRegister::R7, regs.r7);
        cpu.reg_write(ArmRegister::R8, regs.r8);
        cpu.reg_write(ArmRegister::SB, regs.sb);
        cpu.reg_write(ArmRegister::SL, regs.sl);
        cpu.reg_write(ArmRegister::FP, regs.fp);
        cpu.reg_write(ArmRegister::IP, regs.ip);
        cpu.reg_write(ArmRegister::SP, regs.sp);
        cpu.reg_write(ArmRegister::LR, regs.lr);
        cpu.reg_write(ArmRegister::PC, regs.pc);
        cpu.reg_write(ArmRegister::Cpsr, regs.cpsr);
    }

    pub(crate) fn read_memory(&self, start_addr: u32, data: &mut [u8]) -> wie_util::Result<usize> {
        let result = self.cpu.lock().mem_read(start_addr, data.len(), data)?;
        let breakpoints = self.breakpoints.lock();
        overlay_breakpoint_originals(&breakpoints, start_addr, data, true);

        Ok(result)
    }

    pub(crate) fn write_memory(&self, start_addr: u32, data: &[u8]) -> wie_util::Result<()> {
        let mut breakpoints = self.breakpoints.lock();
        let write_end = start_addr + data.len() as u32;

        update_breakpoint_originals(&mut breakpoints, start_addr, data);

        let mut cpu = self.cpu.lock();
        cpu.mem_write(start_addr, data)?;

        for (&addr, breakpoint) in breakpoints.iter() {
            if breakpoint.patched {
                let breakpoint_end = addr + breakpoint.kind.original_len() as u32;
                if addr < write_end && breakpoint_end > start_addr {
                    cpu.mem_write(addr, &breakpoint.kind.patch_bytes()[..breakpoint.kind.patch_len()])?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn recv_stop_event_timeout(&self, timeout: Duration) -> Result<DebugStopReason, channel::RecvTimeoutError> {
        self.stop_event_rx.recv_timeout(timeout)
    }

    pub(crate) fn pause(&self) {
        self.interrupt();
        while !matches!(*self.run_state.lock(), RunState::Paused) {
            self.stop_event_rx.recv().unwrap();
        }
        // The initial stop is reported by gdbstub during the connection handshake.
        while self.stop_event_rx.try_recv().is_ok() {}
    }

    pub(crate) fn interrupt(&self) {
        let mut state = self.run_state.lock();
        if matches!(*state, RunState::Running(_)) {
            *state = RunState::Interrupt;
        }
    }

    pub(crate) fn resume(&self, step_thread: Option<ThreadId>, resumed_threads: Option<Vec<ThreadId>>) {
        *self.resumed_threads.lock() = resumed_threads;
        *self.run_state.lock() = RunState::Running(step_thread.map_or(ResumeMode::Continue, ResumeMode::Step));
        let _ = self.resume_tx.try_send(());
    }

    pub(crate) fn is_thread_resumed(&self, thread_id: ThreadId) -> bool {
        self.resumed_threads.lock().as_ref().is_none_or(|threads| threads.contains(&thread_id))
    }

    pub(crate) fn detach(&self) -> wie_util::Result<()> {
        self.pause();
        let addresses: Vec<_> = self.breakpoints.lock().keys().copied().collect();
        for address in addresses {
            self.remove_breakpoint(address)?;
        }
        self.resume(None, None);
        Ok(())
    }

    pub(crate) fn add_breakpoint(&self, addr: u32, kind: DebugBreakpointKind) -> wie_util::Result<()> {
        let addr = Self::normalize_addr(addr);
        if self.breakpoints.lock().contains_key(&addr) {
            return Ok(());
        }

        let original_len = kind.original_len();
        let patch_len = kind.patch_len();
        let mut original = [0; 4];
        {
            let mut cpu = self.cpu.lock();
            cpu.mem_read(addr, original_len, &mut original[..original_len])?;
            cpu.mem_write(addr, &kind.patch_bytes()[..patch_len])?;
        }

        self.breakpoints.lock().insert(
            addr,
            SoftwareBreakpoint {
                kind,
                original,
                patched: true,
            },
        );

        Ok(())
    }

    pub(crate) fn remove_breakpoint(&self, addr: u32) -> wie_util::Result<()> {
        let addr = Self::normalize_addr(addr);
        let breakpoint = self.breakpoints.lock().remove(&addr);

        if let Some(breakpoint) = breakpoint
            && breakpoint.patched
        {
            self.cpu.lock().mem_write(addr, &breakpoint.original[..breakpoint.kind.original_len()])?;
        }

        Ok(())
    }

    pub(crate) fn current_thread(&self) -> Option<ThreadId> {
        *self.current_thread.lock()
    }

    pub(crate) fn has_breakpoints(&self) -> bool {
        !self.breakpoints.lock().is_empty()
    }

    pub(crate) fn on_thread_entered(&self, thread_id: ThreadId) {
        *self.current_thread.lock() = Some(thread_id);
        if !matches!(*self.run_state.lock(), RunState::Initial) {
            self.wait_for_resume_mode(thread_id);
        }
    }

    pub(crate) fn on_thread_exited(&self, thread_id: ThreadId) {
        let mut current_thread = self.current_thread.lock();
        if *current_thread == Some(thread_id) {
            *current_thread = None;
        }
    }

    fn wait_for_resume_mode(&self, thread_id: ThreadId) -> ResumeMode {
        loop {
            let state = *self.run_state.lock();
            match state {
                RunState::Running(mode) => return mode,
                RunState::Paused => self.resume_rx.recv().unwrap(),
                RunState::Initial | RunState::Interrupt => {
                    self.stop(DebugStopReason::Signal(DebugSignal::Int, thread_id));
                }
            }
        }
    }

    fn stop(&self, reason: DebugStopReason) {
        let mut state = self.run_state.lock();
        *state = RunState::Paused;
        self.stop_event_tx.send(reason).unwrap();
    }

    fn normalize_addr(addr: u32) -> u32 {
        if addr & 1 == 1 { addr - 1 } else { addr }
    }

    fn try_restore_breakpoint(&self, addr: u32) -> wie_util::Result<bool> {
        let mut cpu = self.cpu.lock();
        let mut breakpoints = self.breakpoints.lock();

        if let Some(breakpoint) = breakpoints.get_mut(&addr)
            && breakpoint.patched
        {
            cpu.mem_write(addr, &breakpoint.original[..breakpoint.kind.original_len()])?;
            breakpoint.patched = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn reinsert_breakpoint(&self, addr: u32) -> wie_util::Result<()> {
        let mut cpu = self.cpu.lock();
        let mut breakpoints = self.breakpoints.lock();

        if let Some(breakpoint) = breakpoints.get_mut(&addr)
            && !breakpoint.patched
        {
            cpu.mem_write(addr, &breakpoint.kind.patch_bytes()[..breakpoint.kind.patch_len()])?;
            breakpoint.patched = true;
        }

        Ok(())
    }

    fn map_stop_reason(error: WieError, thread_id: ThreadId) -> DebugStopReason {
        let signal = match error {
            WieError::AllocationFailure => DebugSignal::Kill,
            WieError::InvalidMemoryAccess(_) => DebugSignal::Segv,
            WieError::Unimplemented(_) => DebugSignal::Sys,
            WieError::JavaException(_) | WieError::JavaExceptionUnwind { .. } => DebugSignal::Trap,
            WieError::FatalError(_) => DebugSignal::Abrt,
        };
        DebugStopReason::Signal(signal, thread_id)
    }
}

pub struct DebuggedArm32CpuEngine {
    debug: Arc<DebugInner>,
    pending_breakpoint_reinsert: Option<u32>,
    step_after_svc: Option<ThreadId>,
}

impl DebuggedArm32CpuEngine {
    pub fn new() -> Self {
        let debug = DebugInner::new();

        Self {
            debug,
            pending_breakpoint_reinsert: None,
            step_after_svc: None,
        }
    }

    pub(crate) fn debug_inner(&self) -> Arc<DebugInner> {
        self.debug.clone()
    }

    fn stop_thread_id(&self) -> ThreadId {
        self.debug.current_thread().unwrap_or(1)
    }
}

impl ArmEngine for DebuggedArm32CpuEngine {
    fn run(&mut self, end: u32, count: u32) -> wie_util::Result<EngineRunResult> {
        let mut instructions_executed = 0;
        loop {
            let resume_mode = self.debug.wait_for_resume_mode(self.stop_thread_id());
            if !self.debug.is_thread_resumed(self.stop_thread_id()) {
                return Ok(EngineRunResult {
                    stop_reason: EngineStopReason::Yield,
                    instructions_executed,
                });
            }
            let stepping = matches!(resume_mode, ResumeMode::Step(thread_id) if Some(thread_id) == self.debug.current_thread());

            if stepping && self.step_after_svc == self.debug.current_thread() {
                self.step_after_svc = None;
                self.debug.stop(DebugStopReason::DoneStep(self.stop_thread_id()));
                continue;
            }
            if !matches!(resume_mode, ResumeMode::Step(thread_id) if Some(thread_id) == self.step_after_svc) {
                self.step_after_svc = None;
            }

            if instructions_executed == count {
                let mut result = self.debug.cpu.lock().run(end, 0)?;
                result.instructions_executed += instructions_executed;
                return Ok(result);
            }

            let result = if let Some(addr) = self.pending_breakpoint_reinsert.take() {
                let result = self.debug.cpu.lock().run(end, 1);
                if let Err(error) = self.debug.reinsert_breakpoint(addr) {
                    self.debug.stop(DebugStopReason::Signal(DebugSignal::Abrt, self.stop_thread_id()));
                    return Err(error);
                }
                result
            } else {
                let current_pc = DebugInner::normalize_addr(self.debug.cpu.lock().reg_read(ArmRegister::PC));

                match self.debug.try_restore_breakpoint(current_pc) {
                    Ok(true) => {
                        self.pending_breakpoint_reinsert = Some(current_pc);
                        self.debug.stop(DebugStopReason::SwBreak(self.stop_thread_id()));
                        continue;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        self.debug.stop(DebugStopReason::Signal(DebugSignal::Abrt, self.stop_thread_id()));
                        return Err(error);
                    }
                }

                let run_count = if !stepping && !self.debug.has_breakpoints() {
                    count - instructions_executed
                } else {
                    1
                };
                self.debug.cpu.lock().run(end, run_count)
            };

            match result {
                Ok(mut result) => {
                    let executed_instruction = result.instructions_executed != 0;
                    instructions_executed += result.instructions_executed;
                    result.instructions_executed = instructions_executed;
                    match result.stop_reason {
                        EngineStopReason::Svc { .. } => {
                            if stepping {
                                self.step_after_svc = self.debug.current_thread();
                            }
                            return Ok(result);
                        }
                        _ if stepping && executed_instruction => self.debug.stop(DebugStopReason::DoneStep(self.stop_thread_id())),
                        EngineStopReason::Yield if instructions_executed < count => continue,
                        _ => return Ok(result),
                    }
                }
                Err(error) => self.debug.stop(DebugInner::map_stop_reason(error, self.stop_thread_id())),
            }
        }
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
        self.debug.cpu.lock().reg_write(reg, value)
    }

    fn reg_read(&self, reg: ArmRegister) -> u32 {
        self.debug.cpu.lock().reg_read(reg)
    }

    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission) {
        self.debug.cpu.lock().mem_map(address, size, permission)
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        self.debug.cpu.lock().mem_write(address, data)
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> wie_util::Result<usize> {
        self.debug.cpu.lock().mem_read(address, size, result)
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.debug.cpu.lock().is_mapped(address, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_past_a_return_boundary_does_not_report_an_empty_step() {
        extern crate std;

        let mut engine = DebuggedArm32CpuEngine::new();
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x2001);
        engine.debug.on_thread_entered(1);
        engine.debug.resume(Some(1), None);
        let debug = engine.debug.clone();
        let (tx, rx) = channel::bounded(1);
        let runner = std::thread::spawn(move || tx.send(engine.run(0x2000, 1)).unwrap());
        let result = rx.recv_timeout(Duration::from_secs(1));
        debug.resume(None, None);
        runner.join().unwrap();
        let result = result.unwrap().unwrap();
        assert!(matches!(result.stop_reason, EngineStopReason::End));
        assert_eq!(result.instructions_executed, 0);
        assert!(debug.stop_event_rx.is_empty());
    }

    #[test]
    fn stepping_svc_stops_before_the_next_guest_instruction() {
        extern crate std;

        let mut engine = DebuggedArm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0x01, 0xdf, 0x01, 0x30]).unwrap(); // svc #1; add r0, #1
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.debug.on_thread_entered(1);
        engine.debug.resume(Some(1), None);
        let result = engine.run(0, 2).unwrap();
        let EngineStopReason::Svc { lr, spsr, .. } = result.stop_reason else {
            panic!("expected SVC");
        };
        engine.reg_write(ArmRegister::Cpsr, spsr);
        engine.reg_write(ArmRegister::PC, lr);

        let debug = engine.debug.clone();
        let runner = std::thread::spawn(move || engine.run(0, 1));
        let stopped = debug.recv_stop_event_timeout(Duration::from_secs(1));
        let context = debug.read_registers();
        debug.resume(None, None);
        runner.join().unwrap().unwrap();
        assert!(stopped.is_ok());
        assert_eq!(context.pc, 0x1002);
        assert_eq!(context.r0, 0);
    }

    #[test]
    fn breakpoint_steps_count_toward_the_svc_instruction_budget() {
        extern crate std;

        let mut engine = DebuggedArm32CpuEngine::new();
        engine.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0x01, 0x30, 0x01, 0x30, 0x01, 0xdf]).unwrap(); // add r0, #1; add r0, #1; svc #1
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1001);
        engine.debug.add_breakpoint(0x1002, DebugBreakpointKind::Thumb16).unwrap();
        let debug = engine.debug.clone();
        debug.resume(None, None);
        let runner = std::thread::spawn(move || {
            let result = engine.run(0, 3).unwrap();
            (result, engine.reg_read(ArmRegister::R0))
        });
        let stopped = debug.recv_stop_event_timeout(Duration::from_secs(1));
        debug.resume(None, None);
        let (result, r0) = runner.join().unwrap();
        assert!(matches!(stopped, Ok(DebugStopReason::SwBreak(1))));
        assert!(matches!(result.stop_reason, EngineStopReason::Svc { category: 1, lr: 0x1006, .. }));
        assert_eq!(result.instructions_executed, 3);
        assert_eq!(r0, 2);
    }

    #[test]
    fn test_thumb_breakpoint_patch_and_restore() {
        let debug = DebugInner::new();
        {
            let mut cpu = debug.cpu.lock();
            cpu.mem_map(0x1000, 0x1000, MemoryPermission::ReadWriteExecute);
            cpu.mem_write(0x1000, &[0x12, 0x34]).unwrap();
        }

        debug.add_breakpoint(0x1001, DebugBreakpointKind::Thumb16).unwrap();

        let mut data = [0; 2];
        debug.read_memory(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x12, 0x34]);

        assert!(debug.try_restore_breakpoint(0x1000).unwrap());
        debug.read_memory(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x12, 0x34]);

        debug.reinsert_breakpoint(0x1000).unwrap();
        debug.read_memory(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x12, 0x34]);

        debug.remove_breakpoint(0x1001).unwrap();
        debug.read_memory(0x1000, &mut data).unwrap();
        assert_eq!(data, [0x12, 0x34]);
    }

    #[test]
    fn test_arm_breakpoint_patch_and_restore() {
        let debug = DebugInner::new();
        {
            let mut cpu = debug.cpu.lock();
            cpu.mem_map(0x2000, 0x1000, MemoryPermission::ReadWriteExecute);
            cpu.mem_write(0x2000, &[0x11, 0x22, 0x33, 0x44]).unwrap();
        }

        debug.add_breakpoint(0x2000, DebugBreakpointKind::Arm32).unwrap();

        let mut data = [0; 4];
        debug.read_memory(0x2000, &mut data).unwrap();
        assert_eq!(data, [0x11, 0x22, 0x33, 0x44]);

        debug.remove_breakpoint(0x2000).unwrap();
        debug.read_memory(0x2000, &mut data).unwrap();
        assert_eq!(data, [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_write_memory_preserves_trap_bytes() {
        let debug = DebugInner::new();
        {
            let mut cpu = debug.cpu.lock();
            cpu.mem_map(0x3000, 0x1000, MemoryPermission::ReadWriteExecute);
            cpu.mem_write(0x3000, &[0xAA, 0xBB, 0xCC, 0xDD]).unwrap();
        }

        debug.add_breakpoint(0x3000, DebugBreakpointKind::Thumb16).unwrap();

        debug.write_memory(0x3000, &[0x11, 0x22]).unwrap();

        let mut data = [0; 2];
        debug.read_memory(0x3000, &mut data).unwrap();
        assert_eq!(data, [0x11, 0x22]);

        let mut raw = [0; 2];
        debug.cpu.lock().mem_read(0x3000, 2, &mut raw).unwrap();
        assert_eq!(raw, [0x00, 0xbe]);

        debug.remove_breakpoint(0x3000).unwrap();
        let mut final_data = [0; 2];
        debug.cpu.lock().mem_read(0x3000, 2, &mut final_data).unwrap();
        assert_eq!(final_data, [0x11, 0x22]);
    }
}
