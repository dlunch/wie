use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::{ops::Range, time::Duration};

use crossbeam::channel;
use spin::Mutex;
use wie_util::WieError;

use crate::{ThreadId, context::ArmCoreContext};

use super::{Arm32CpuEngine, ArmEngine, ArmRegister, MemoryPermission};

#[derive(Copy, Clone)]
enum ResumeMode {
    Continue,
    Step,
}

#[derive(Copy, Clone)]
enum RunState {
    Paused,
    Running(ResumeMode),
}

#[derive(Copy, Clone)]
pub(crate) enum DebugSignal {
    Kill,
    Segv,
    Sys,
    Trap,
    Abrt,
}

#[derive(Copy, Clone)]
pub(crate) enum DebugStopReason {
    Signal(DebugSignal),
    SwBreak(ThreadId),
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
    resume_tx: channel::Sender<ResumeMode>,
    resume_rx: channel::Receiver<ResumeMode>,
    interrupt_pending: Mutex<bool>,
    breakpoints: Mutex<BTreeMap<u32, SoftwareBreakpoint>>,
    active_threads: Mutex<Vec<ThreadId>>,
    current_thread: Mutex<Option<ThreadId>>,
    thread_ready_tx: channel::Sender<()>,
    thread_ready_rx: channel::Receiver<()>,
}

impl DebugInner {
    fn new() -> Arc<Self> {
        let (stop_event_tx, stop_event_rx) = channel::unbounded();
        let (resume_tx, resume_rx) = channel::unbounded();
        let (thread_ready_tx, thread_ready_rx) = channel::unbounded();

        Arc::new(Self {
            cpu: Mutex::new(Arm32CpuEngine::new()),
            stop_event_tx,
            stop_event_rx,
            resume_tx,
            resume_rx,
            interrupt_pending: Mutex::new(false),
            breakpoints: Mutex::new(BTreeMap::new()),
            active_threads: Mutex::new(Vec::new()),
            current_thread: Mutex::new(None),
            thread_ready_tx,
            thread_ready_rx,
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

    pub(crate) fn wait_for_thread_ready(&self) {
        let _ = self.thread_ready_rx.recv();
    }

    pub(crate) fn resume_continue(&self) {
        self.resume_tx.send(ResumeMode::Continue).unwrap();
    }

    pub(crate) fn resume_step(&self) {
        self.resume_tx.send(ResumeMode::Step).unwrap();
    }

    pub(crate) fn interrupt(&self) {
        *self.interrupt_pending.lock() = true;
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

    pub(crate) fn active_threads(&self) -> Vec<ThreadId> {
        self.active_threads.lock().clone()
    }

    pub(crate) fn current_thread(&self) -> Option<ThreadId> {
        *self.current_thread.lock()
    }

    pub(crate) fn has_breakpoints(&self) -> bool {
        !self.breakpoints.lock().is_empty()
    }

    pub(crate) fn on_thread_created(&self, thread_id: ThreadId) {
        {
            let mut active_threads = self.active_threads.lock();
            if !active_threads.contains(&thread_id) {
                active_threads.push(thread_id);
            }
        }

        let _ = self.thread_ready_tx.send(());
    }

    pub(crate) fn on_thread_deleted(&self, thread_id: ThreadId) {
        self.active_threads.lock().retain(|&x| x != thread_id);

        let mut current_thread = self.current_thread.lock();
        if *current_thread == Some(thread_id) {
            *current_thread = None;
        }
    }

    pub(crate) fn on_thread_entered(&self, thread_id: ThreadId) {
        *self.current_thread.lock() = Some(thread_id);
    }

    pub(crate) fn on_thread_exited(&self, thread_id: ThreadId) {
        let mut current_thread = self.current_thread.lock();
        if *current_thread == Some(thread_id) {
            *current_thread = None;
        }
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

    fn take_interrupt(&self) -> bool {
        let mut interrupt_pending = self.interrupt_pending.lock();
        let result = *interrupt_pending;
        *interrupt_pending = false;

        result
    }

    fn wait_for_resume(&self) -> ResumeMode {
        self.resume_rx.recv().expect("resume channel disconnected")
    }

    fn map_stop_reason(error: WieError) -> DebugStopReason {
        match error {
            WieError::AllocationFailure => DebugStopReason::Signal(DebugSignal::Kill),
            WieError::InvalidMemoryAccess(_) => DebugStopReason::Signal(DebugSignal::Segv),
            WieError::Unimplemented(_) => DebugStopReason::Signal(DebugSignal::Sys),
            WieError::JavaException(_) => DebugStopReason::Signal(DebugSignal::Trap),
            WieError::JavaExceptionUnwind { .. } => DebugStopReason::Signal(DebugSignal::Trap),
            WieError::FatalError(_) => DebugStopReason::Signal(DebugSignal::Abrt),
        }
    }
}

pub struct DebuggedArm32CpuEngine {
    debug: Arc<DebugInner>,
    run_state: RunState,
    pending_breakpoint_reinsert: Option<u32>,
}

impl DebuggedArm32CpuEngine {
    pub fn new() -> Self {
        let debug = DebugInner::new();

        Self {
            debug,
            run_state: RunState::Paused,
            pending_breakpoint_reinsert: None,
        }
    }

    pub(crate) fn debug_inner(&self) -> Arc<DebugInner> {
        self.debug.clone()
    }

    fn wait_for_resume_mode(&mut self) -> ResumeMode {
        match self.run_state {
            RunState::Running(mode) => mode,
            RunState::Paused => {
                let mode = self.debug.wait_for_resume();
                self.run_state = RunState::Running(mode);
                mode
            }
        }
    }

    fn stop(&mut self, reason: DebugStopReason) {
        self.debug.stop_event_tx.send(reason).unwrap();
        self.run_state = RunState::Paused;
    }

    fn stop_thread_id(&self) -> ThreadId {
        self.debug.current_thread().unwrap_or(1)
    }

    fn handle_breakpoint_reinsert(&mut self, addr: u32, end: u32, hook: &Range<u32>, resume_mode: ResumeMode) -> wie_util::Result<()> {
        let result = self.debug.cpu.lock().run(end, hook, 1);
        if let Err(error) = self.debug.reinsert_breakpoint(addr) {
            self.stop(DebugStopReason::Signal(DebugSignal::Abrt));
            return Err(error);
        }

        match result {
            Ok(_) if matches!(resume_mode, ResumeMode::Continue) => {}
            Ok(_) => self.stop(DebugStopReason::SwBreak(self.stop_thread_id())),
            Err(error) => self.stop(DebugInner::map_stop_reason(error)),
        }

        Ok(())
    }
}

impl ArmEngine for DebuggedArm32CpuEngine {
    fn run(&mut self, end: u32, hook: &Range<u32>, count: u32) -> wie_util::Result<u32> {
        loop {
            if self.debug.take_interrupt() {
                self.stop(DebugStopReason::Signal(DebugSignal::Trap));
                continue;
            }

            let resume_mode = self.wait_for_resume_mode();

            if let Some(addr) = self.pending_breakpoint_reinsert.take() {
                self.handle_breakpoint_reinsert(addr, end, hook, resume_mode)?;
                continue;
            }

            let current_pc = DebugInner::normalize_addr(self.debug.cpu.lock().reg_read(ArmRegister::PC));

            match self.debug.try_restore_breakpoint(current_pc) {
                Ok(true) => {
                    self.pending_breakpoint_reinsert = Some(current_pc);
                    self.stop(DebugStopReason::SwBreak(self.stop_thread_id()));
                    continue;
                }
                Ok(false) => {}
                Err(error) => {
                    self.stop(DebugStopReason::Signal(DebugSignal::Abrt));
                    return Err(error);
                }
            }

            let run_count = match resume_mode {
                ResumeMode::Continue => {
                    if self.debug.has_breakpoints() {
                        1
                    } else {
                        count
                    }
                }
                ResumeMode::Step => 1,
            };

            let result = self.debug.cpu.lock().run(end, hook, run_count);

            match result {
                Ok(pc) => match resume_mode {
                    ResumeMode::Continue => return Ok(pc),
                    ResumeMode::Step => self.stop(DebugStopReason::SwBreak(self.stop_thread_id())),
                },
                Err(error) => self.stop(DebugInner::map_stop_reason(error)),
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
