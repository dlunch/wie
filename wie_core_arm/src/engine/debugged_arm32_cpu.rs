use alloc::{collections::BTreeSet, sync::Arc, vec::Vec};
use core::ops::Range;

use crossbeam::channel;
use gdbstub::{
    common::{Signal, Tid},
    stub::MultiThreadStopReason,
};
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

pub(crate) struct DebugInner {
    cpu: Mutex<Arm32CpuEngine>,
    stop_event_tx: channel::Sender<MultiThreadStopReason<u32>>,
    pub(crate) stop_event_rx: channel::Receiver<MultiThreadStopReason<u32>>,
    resume_tx: channel::Sender<ResumeMode>,
    resume_rx: channel::Receiver<ResumeMode>,
    interrupt_pending: Mutex<bool>,
    breakpoints: Mutex<BTreeSet<u32>>,
    active_threads: Mutex<Vec<ThreadId>>,
    current_thread: Mutex<Option<ThreadId>>,
    thread_ready_tx: channel::Sender<()>,
    pub(crate) thread_ready_rx: channel::Receiver<()>,
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
            breakpoints: Mutex::new(BTreeSet::new()),
            active_threads: Mutex::new(Vec::new()),
            current_thread: Mutex::new(None),
            thread_ready_tx,
            thread_ready_rx,
        })
    }

    pub(crate) fn read_registers(&self) -> [u32; 17] {
        let cpu = self.cpu.lock();

        [
            cpu.reg_read(ArmRegister::R0),
            cpu.reg_read(ArmRegister::R1),
            cpu.reg_read(ArmRegister::R2),
            cpu.reg_read(ArmRegister::R3),
            cpu.reg_read(ArmRegister::R4),
            cpu.reg_read(ArmRegister::R5),
            cpu.reg_read(ArmRegister::R6),
            cpu.reg_read(ArmRegister::R7),
            cpu.reg_read(ArmRegister::R8),
            cpu.reg_read(ArmRegister::SB),
            cpu.reg_read(ArmRegister::SL),
            cpu.reg_read(ArmRegister::FP),
            cpu.reg_read(ArmRegister::IP),
            cpu.reg_read(ArmRegister::SP),
            cpu.reg_read(ArmRegister::LR),
            cpu.reg_read(ArmRegister::PC),
            cpu.reg_read(ArmRegister::Cpsr),
        ]
    }

    pub(crate) fn write_registers(&self, regs: [u32; 17]) {
        let mut cpu = self.cpu.lock();

        cpu.reg_write(ArmRegister::R0, regs[0]);
        cpu.reg_write(ArmRegister::R1, regs[1]);
        cpu.reg_write(ArmRegister::R2, regs[2]);
        cpu.reg_write(ArmRegister::R3, regs[3]);
        cpu.reg_write(ArmRegister::R4, regs[4]);
        cpu.reg_write(ArmRegister::R5, regs[5]);
        cpu.reg_write(ArmRegister::R6, regs[6]);
        cpu.reg_write(ArmRegister::R7, regs[7]);
        cpu.reg_write(ArmRegister::R8, regs[8]);
        cpu.reg_write(ArmRegister::SB, regs[9]);
        cpu.reg_write(ArmRegister::SL, regs[10]);
        cpu.reg_write(ArmRegister::FP, regs[11]);
        cpu.reg_write(ArmRegister::IP, regs[12]);
        cpu.reg_write(ArmRegister::SP, regs[13]);
        cpu.reg_write(ArmRegister::LR, regs[14]);
        cpu.reg_write(ArmRegister::PC, regs[15]);
        cpu.reg_write(ArmRegister::Cpsr, regs[16]);
    }

    pub(crate) fn read_memory(&self, start_addr: u32, data: &mut [u8]) -> wie_util::Result<usize> {
        self.cpu.lock().mem_read(start_addr, data.len(), data)
    }

    pub(crate) fn write_memory(&self, start_addr: u32, data: &[u8]) -> wie_util::Result<()> {
        self.cpu.lock().mem_write(start_addr, data)
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

    pub(crate) fn add_breakpoint(&self, addr: u32) {
        self.breakpoints.lock().insert(Self::normalize_addr(addr));
    }

    pub(crate) fn remove_breakpoint(&self, addr: u32) {
        self.breakpoints.lock().remove(&Self::normalize_addr(addr));
    }

    pub(crate) fn active_threads(&self) -> Vec<ThreadId> {
        self.active_threads.lock().clone()
    }

    pub(crate) fn current_thread(&self) -> Option<ThreadId> {
        *self.current_thread.lock()
    }

    pub(crate) fn on_thread_created(&self, thread_id: ThreadId, context: &ArmCoreContext) {
        let mut active_threads = self.active_threads.lock();
        if !active_threads.contains(&thread_id) {
            active_threads.push(thread_id);
        }
        drop(active_threads);

        self.write_registers([
            context.r0,
            context.r1,
            context.r2,
            context.r3,
            context.r4,
            context.r5,
            context.r6,
            context.r7,
            context.r8,
            context.sb,
            context.sl,
            context.fp,
            context.ip,
            context.sp,
            context.lr,
            context.pc,
            context.cpsr,
        ]);

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

    fn take_interrupt(&self) -> bool {
        let mut interrupt_pending = self.interrupt_pending.lock();
        let result = *interrupt_pending;
        *interrupt_pending = false;

        result
    }

    fn wait_for_resume(&self) -> ResumeMode {
        loop {
            if let Ok(mode) = self.resume_rx.recv() {
                return mode;
            }
        }
    }

    pub(crate) fn enqueue_stop_reason(&self, reason: MultiThreadStopReason<u32>) {
        self.stop_event_tx.send(reason).unwrap();
    }

    fn map_stop_reason(error: WieError) -> MultiThreadStopReason<u32> {
        match error {
            WieError::AllocationFailure => MultiThreadStopReason::Signal(Signal::SIGKILL),
            WieError::InvalidMemoryAccess(_) => MultiThreadStopReason::Signal(Signal::SIGSEGV),
            WieError::Unimplemented(_) => MultiThreadStopReason::Signal(Signal::SIGSYS),
            WieError::JavaException(_) => MultiThreadStopReason::Signal(Signal::SIGTRAP),
            WieError::JavaExceptionUnwind { .. } => MultiThreadStopReason::Signal(Signal::SIGTRAP),
            WieError::FatalError(_) => MultiThreadStopReason::Signal(Signal::SIGABRT),
        }
    }
}

pub struct DebuggedArm32CpuEngine {
    debug: Arc<DebugInner>,
    run_state: RunState,
    stopped_on_breakpoint: Option<u32>,
}

impl DebuggedArm32CpuEngine {
    pub fn new() -> Self {
        let debug = DebugInner::new();

        Self {
            debug,
            run_state: RunState::Paused,
            stopped_on_breakpoint: None,
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

    fn stop(&mut self, reason: MultiThreadStopReason<u32>) {
        self.debug.enqueue_stop_reason(reason);
        self.run_state = RunState::Paused;
    }

    fn current_tid(&self) -> Tid {
        let thread_id = self.debug.current_thread().unwrap_or(1);
        Tid::try_from(thread_id).unwrap()
    }
}

impl ArmEngine for DebuggedArm32CpuEngine {
    fn run(&mut self, end: u32, hook: &Range<u32>, count: u32) -> wie_util::Result<u32> {
        loop {
            if self.debug.take_interrupt() {
                self.stop(MultiThreadStopReason::SwBreak(self.current_tid()));
                continue;
            }

            let resume_mode = self.wait_for_resume_mode();

            let current_pc = {
                let cpu = self.debug.cpu.lock();
                DebugInner::normalize_addr(cpu.reg_read(ArmRegister::PC))
            };

            if self.stopped_on_breakpoint == Some(current_pc) {
                self.stopped_on_breakpoint = None;
            } else if self.debug.breakpoints.lock().contains(&current_pc) {
                self.stopped_on_breakpoint = Some(current_pc);
                self.stop(MultiThreadStopReason::Signal(Signal::SIGTRAP));
                continue;
            }

            let run_count = match resume_mode {
                ResumeMode::Continue => count,
                ResumeMode::Step => 1,
            };

            let result = {
                let mut cpu = self.debug.cpu.lock();
                cpu.run(end, hook, run_count)
            };

            match result {
                Ok(pc) => match resume_mode {
                    ResumeMode::Continue => return Ok(pc),
                    ResumeMode::Step => self.stop(MultiThreadStopReason::SwBreak(self.current_tid())),
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
