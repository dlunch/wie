#[cfg(any(target_arch = "wasm32", test))]
mod dummy;
#[cfg(not(target_arch = "wasm32"))]
mod tcp;

#[cfg(target_arch = "wasm32")]
pub(crate) use dummy::start;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use tcp::start;

use alloc::{sync::Arc, vec::Vec};
use core::{marker::PhantomData, time::Duration};

use crossbeam::channel;
use gdbstub::{
    arch::Arch,
    common::{Pid, Signal, Tid},
    conn::ConnectionExt,
    stub::{
        MultiThreadStopReason,
        run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError},
    },
    target::{
        Target, TargetError, TargetResult,
        ext::base::{
            BaseOps,
            multithread::{
                MultiThreadBase, MultiThreadResume, MultiThreadResumeOps, MultiThreadSchedulerLocking, MultiThreadSchedulerLockingOps,
                MultiThreadSingleStep, MultiThreadSingleStepOps,
            },
        },
        ext::breakpoints::{Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps},
        ext::extended_mode::{Args, AttachKind, CurrentActivePid, CurrentActivePidOps, ExtendedMode, ExtendedModeOps, ShouldTerminate},
    },
};
use gdbstub_arch::arm::{ArmBreakpointKind, Armv4t, reg::ArmCoreRegs};

use crate::{
    ArmCore,
    context::ArmCoreContext,
    engine::{DebugBreakpointKind, DebugInner, DebugSignal, DebugStopReason},
};

type GdbTargetError = &'static str;

fn to_gdb_signal(signal: DebugSignal) -> Signal {
    match signal {
        DebugSignal::Int => Signal::SIGINT,
        DebugSignal::Kill => Signal::SIGKILL,
        DebugSignal::Segv => Signal::SIGSEGV,
        DebugSignal::Sys => Signal::SIGSYS,
        DebugSignal::Trap => Signal::SIGTRAP,
        DebugSignal::Abrt => Signal::SIGABRT,
    }
}

fn to_gdb_stop_reason(reason: DebugStopReason) -> MultiThreadStopReason<u32> {
    match reason {
        DebugStopReason::Signal(signal, thread_id) => MultiThreadStopReason::SignalWithThread {
            signal: to_gdb_signal(signal),
            tid: Tid::try_from(thread_id).unwrap(),
        },
        DebugStopReason::SwBreak(thread_id) => MultiThreadStopReason::SwBreak(Tid::try_from(thread_id).unwrap()),
        DebugStopReason::DoneStep(thread_id) => MultiThreadStopReason::SignalWithThread {
            signal: Signal::SIGTRAP,
            tid: Tid::try_from(thread_id).unwrap(),
        },
    }
}

fn context_to_regs(ctx: &ArmCoreContext, regs: &mut ArmCoreRegs) {
    regs.r = [
        ctx.r0, ctx.r1, ctx.r2, ctx.r3, ctx.r4, ctx.r5, ctx.r6, ctx.r7, ctx.r8, ctx.sb, ctx.sl, ctx.fp, ctx.ip,
    ];
    regs.sp = ctx.sp;
    regs.lr = ctx.lr;
    regs.pc = ctx.pc;
    regs.cpsr = ctx.cpsr;
}

fn regs_to_context(regs: &ArmCoreRegs) -> ArmCoreContext {
    ArmCoreContext {
        r0: regs.r[0],
        r1: regs.r[1],
        r2: regs.r[2],
        r3: regs.r[3],
        r4: regs.r[4],
        r5: regs.r[5],
        r6: regs.r[6],
        r7: regs.r[7],
        r8: regs.r[8],
        sb: regs.r[9],
        sl: regs.r[10],
        fp: regs.r[11],
        ip: regs.r[12],
        sp: regs.sp,
        lr: regs.lr,
        pc: regs.pc,
        cpsr: regs.cpsr,
    }
}

pub struct GdbTarget {
    core: ArmCore,
    debug: Arc<DebugInner>,
    step_threads: Vec<crate::ThreadId>,
    resumed_threads: Vec<crate::ThreadId>,
    scheduler_locked: bool,
}

impl GdbTarget {
    fn new(core: ArmCore) -> Self {
        let debug = core.debug_inner().unwrap();
        Self {
            core,
            debug,
            step_threads: Vec::new(),
            resumed_threads: Vec::new(),
            scheduler_locked: false,
        }
    }
}

impl Target for GdbTarget {
    type Arch = Armv4t;

    type Error = GdbTargetError;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::MultiThread(self)
    }

    #[inline(always)]
    fn guard_rail_implicit_sw_breakpoints(&self) -> bool {
        true
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_extended_mode(&mut self) -> Option<ExtendedModeOps<'_, Self>> {
        Some(self)
    }
}

impl MultiThreadBase for GdbTarget {
    fn support_resume(&mut self) -> Option<MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn read_registers(&mut self, regs: &mut ArmCoreRegs, tid: Tid) -> TargetResult<(), Self> {
        let thread_id = u32::try_from(tid.get()).map_err(|_| TargetError::NonFatal)? as usize;
        if !self.core.get_thread_ids().contains(&thread_id) {
            return Err(TargetError::NonFatal);
        }
        let ctx = if self.debug.current_thread() == Some(thread_id) {
            self.debug.read_registers()
        } else {
            self.core.read_thread_context(thread_id).map_err(|_| TargetError::NonFatal)?
        };

        context_to_regs(&ctx, regs);
        Ok(())
    }

    #[inline(always)]
    fn write_registers(&mut self, regs: &ArmCoreRegs, tid: Tid) -> TargetResult<(), Self> {
        let thread_id = u32::try_from(tid.get()).map_err(|_| TargetError::NonFatal)? as usize;
        if !self.core.get_thread_ids().contains(&thread_id) {
            return Err(TargetError::NonFatal);
        }
        let ctx = regs_to_context(regs);

        if self.debug.current_thread() == Some(thread_id) {
            self.debug.write_registers(&ctx);
        } else {
            self.core.write_thread_context(thread_id, &ctx);
        }

        Ok(())
    }

    #[inline(always)]
    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8], _tid: Tid) -> TargetResult<usize, Self> {
        self.debug.read_memory(start_addr, data).map_err(|_| TargetError::NonFatal)
    }

    #[inline(always)]
    fn write_addrs(&mut self, start_addr: u32, data: &[u8], _tid: Tid) -> TargetResult<(), Self> {
        self.debug.write_memory(start_addr, data).map_err(|_| TargetError::NonFatal)
    }

    #[inline(always)]
    fn list_active_threads(&mut self, thread_is_active: &mut dyn FnMut(Tid)) -> Result<(), Self::Error> {
        let mut thread_ids = self.core.get_thread_ids();
        if let Some(index) = thread_ids.iter().position(|&id| Some(id) == self.debug.current_thread()) {
            thread_ids.swap(0, index);
        }

        for thread_id in thread_ids {
            thread_is_active(Tid::try_from(thread_id).unwrap());
        }

        Ok(())
    }
}

impl MultiThreadResume for GdbTarget {
    fn resume(&mut self) -> Result<(), Self::Error> {
        self.debug.resume(
            core::mem::take(&mut self.step_threads),
            self.scheduler_locked.then(|| core::mem::take(&mut self.resumed_threads)),
        );

        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        self.step_threads.clear();
        self.resumed_threads.clear();
        self.scheduler_locked = false;
        Ok(())
    }

    fn set_resume_action_continue(&mut self, tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resumed_threads.push(tid.get());
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<MultiThreadSingleStepOps<'_, Self>> {
        Some(self)
    }

    fn support_scheduler_locking(&mut self) -> Option<MultiThreadSchedulerLockingOps<'_, Self>> {
        Some(self)
    }
}

impl MultiThreadSchedulerLocking for GdbTarget {
    fn set_resume_action_scheduler_lock(&mut self) -> Result<(), Self::Error> {
        self.scheduler_locked = true;
        Ok(())
    }
}

impl MultiThreadSingleStep for GdbTarget {
    fn set_resume_action_step(&mut self, tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.step_threads.push(tid.get());
        self.resumed_threads.push(tid.get());
        Ok(())
    }
}

impl Breakpoints for GdbTarget {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for GdbTarget {
    fn add_sw_breakpoint(&mut self, addr: u32, kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        let kind = match kind {
            ArmBreakpointKind::Thumb16 => DebugBreakpointKind::Thumb16,
            ArmBreakpointKind::Thumb32 => DebugBreakpointKind::Thumb32,
            ArmBreakpointKind::Arm32 => DebugBreakpointKind::Arm32,
        };

        self.debug.add_breakpoint(addr, kind).map_err(|_| TargetError::NonFatal)?;

        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: u32, _kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        self.debug.remove_breakpoint(addr).map_err(|_| TargetError::NonFatal)?;

        Ok(true)
    }
}

impl ExtendedMode for GdbTarget {
    fn run(&mut self, _filename: Option<&[u8]>, _args: Args<'_, '_>) -> TargetResult<Pid, Self> {
        Err(TargetError::NonFatal)
    }

    fn attach(&mut self, pid: Pid) -> TargetResult<(), Self> {
        if pid.get() != 1 {
            return Err(TargetError::NonFatal);
        }
        self.debug.pause();
        Ok(())
    }

    fn query_if_attached(&mut self, pid: Pid) -> TargetResult<AttachKind, Self> {
        if pid.get() != 1 {
            return Err(TargetError::NonFatal);
        }
        Ok(AttachKind::Attach)
    }

    fn kill(&mut self, _pid: Option<Pid>) -> TargetResult<ShouldTerminate, Self> {
        Err(TargetError::NonFatal)
    }

    fn restart(&mut self) -> Result<(), Self::Error> {
        Err("Restarting the emulator through GDB is not supported")
    }

    fn support_current_active_pid(&mut self) -> Option<CurrentActivePidOps<'_, Self>> {
        Some(self)
    }
}

impl CurrentActivePid for GdbTarget {
    fn current_active_pid(&mut self) -> Result<Pid, Self::Error> {
        Ok(Pid::new(1).unwrap())
    }
}

struct GdbBlockingEventLoop<C>(PhantomData<C>);

impl<C: ConnectionExt> BlockingEventLoop for GdbBlockingEventLoop<C> {
    type Target = GdbTarget;
    type Connection = C;

    type StopReason = MultiThreadStopReason<u32>;

    fn wait_for_stop_reason(
        target: &mut GdbTarget,
        conn: &mut Self::Connection,
    ) -> Result<Event<MultiThreadStopReason<u32>>, WaitForStopReasonError<GdbTargetError, C::Error>> {
        loop {
            match target.debug.recv_stop_event_timeout(Duration::from_millis(10)) {
                Ok(reason) => return Ok(Event::TargetStopped(to_gdb_stop_reason(reason))),
                Err(channel::RecvTimeoutError::Timeout) => match conn.peek() {
                    Ok(Some(_)) => match conn.read() {
                        Ok(byte) => return Ok(Event::IncomingData(byte)),
                        Err(error) => return Err(WaitForStopReasonError::Connection(error)),
                    },
                    Ok(None) => {}
                    Err(error) => return Err(WaitForStopReasonError::Connection(error)),
                },
                Err(_) => return Err(WaitForStopReasonError::Target("Target error")),
            }
        }
    }

    fn on_interrupt(target: &mut GdbTarget) -> Result<Option<MultiThreadStopReason<u32>>, GdbTargetError> {
        target.debug.interrupt();
        Ok(None::<MultiThreadStopReason<u32>>)
    }
}
