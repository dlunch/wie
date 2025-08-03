extern crate std;

use alloc::sync::Arc;
use core::ops::Range;
use wie_util::WieError;

use crossbeam::channel;
use gdbstub::{common::Signal, stub::MultiThreadStopReason};

use super::{Arm32CpuEngine, ArmEngine};

pub struct DebuggedArm32CpuEngine {
    arm32_cpu_engine: Arm32CpuEngine,
    pub(crate) stop_event_tx: Arc<channel::Sender<MultiThreadStopReason<u32>>>,
    pub(crate) stop_event_rx: Arc<channel::Receiver<MultiThreadStopReason<u32>>>,
    pub(crate) resume_event_tx: Arc<channel::Sender<()>>,
    pub(crate) resume_event_rx: Arc<channel::Receiver<()>>,
}

impl DebuggedArm32CpuEngine {
    pub fn new() -> Self {
        let (stop_event_tx, stop_event_rx) = channel::unbounded();
        let (resume_event_tx, resume_event_rx) = channel::unbounded();

        Self {
            arm32_cpu_engine: Arm32CpuEngine::new(),
            stop_event_tx: Arc::new(stop_event_tx),
            stop_event_rx: Arc::new(stop_event_rx),
            resume_event_tx: Arc::new(resume_event_tx),
            resume_event_rx: Arc::new(resume_event_rx),
        }
    }
}

impl ArmEngine for DebuggedArm32CpuEngine {
    fn run(&mut self, end: u32, hook: &Range<u32>, count: u32) -> wie_util::Result<u32> {
        loop {
            let result = self.arm32_cpu_engine.run(end, hook, count);

            if let Err(x) = result {
                let reason = match x {
                    WieError::AllocationFailure => MultiThreadStopReason::Signal(Signal::SIGKILL),
                    WieError::InvalidMemoryAccess(_) => MultiThreadStopReason::Signal(Signal::SIGSEGV),
                    WieError::Unimplemented(_) => MultiThreadStopReason::Signal(Signal::SIGSYS),
                    WieError::JavaException(_) => MultiThreadStopReason::Signal(Signal::SIGTRAP),
                    WieError::FatalError(_) => MultiThreadStopReason::Signal(Signal::SIGABRT),
                };

                self.stop_event_tx.send(reason).unwrap();

                // wait until resume is called
                // TODO this should be async wait
                loop {
                    if self.resume_event_rx.recv().is_ok() {
                        break;
                    }
                }
            } else {
                return result;
            }
        }
    }

    fn reg_write(&mut self, reg: super::ArmRegister, value: u32) {
        self.arm32_cpu_engine.reg_write(reg, value)
    }

    fn reg_read(&self, reg: super::ArmRegister) -> u32 {
        self.arm32_cpu_engine.reg_read(reg)
    }

    fn mem_map(&mut self, address: u32, size: usize, permission: super::MemoryPermission) {
        self.arm32_cpu_engine.mem_map(address, size, permission)
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        self.arm32_cpu_engine.mem_write(address, data)
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> wie_util::Result<usize> {
        self.arm32_cpu_engine.mem_read(address, size, result)
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.arm32_cpu_engine.is_mapped(address, size)
    }
}
