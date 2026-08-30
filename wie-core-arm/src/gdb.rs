extern crate std; // we need thread

use alloc::{format, sync::Arc};
use std::{
    io,
    net::{TcpListener, TcpStream},
    println, thread,
    time::Duration,
};

use crossbeam::channel;
use gdbstub::{
    arch::Arch,
    common::{Pid, Signal, Tid},
    conn::ConnectionExt,
    stub::{
        DisconnectReason, GdbStub, MultiThreadStopReason,
        run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError},
    },
    target::{
        Target, TargetError, TargetResult,
        ext::base::{
            BaseOps,
            multithread::{MultiThreadBase, MultiThreadResume, MultiThreadResumeOps, MultiThreadSingleStep, MultiThreadSingleStepOps},
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
        DebugSignal::Kill => Signal::SIGKILL,
        DebugSignal::Segv => Signal::SIGSEGV,
        DebugSignal::Sys => Signal::SIGSYS,
        DebugSignal::Trap => Signal::SIGTRAP,
        DebugSignal::Abrt => Signal::SIGABRT,
    }
}

fn to_gdb_stop_reason(reason: DebugStopReason) -> MultiThreadStopReason<u32> {
    match reason {
        DebugStopReason::Signal(signal) => MultiThreadStopReason::Signal(to_gdb_signal(signal)),
        DebugStopReason::SwBreak(thread_id) => MultiThreadStopReason::SwBreak(Tid::try_from(thread_id).unwrap()),
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
    resume_step: bool,
}

impl GdbTarget {
    pub fn start(core: ArmCore) -> wie_util::Result<()> {
        let debug = {
            let inner = core.inner.lock();

            inner
                .engine
                .as_any()
                .downcast_ref::<crate::engine::DebuggedArm32CpuEngine>()
                .unwrap()
                .debug_inner()
        };

        let (startup_tx, startup_rx) = channel::bounded(1);
        let this = GdbTarget {
            core,
            debug,
            resume_step: false,
        };

        thread::spawn(move || {
            let _ = this.run_gdb_server(startup_tx);
        });

        match startup_rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(wie_util::WieError::FatalError(format!("Failed to start GDB server: {err}"))),
            Err(err) => Err(wie_util::WieError::FatalError(format!("Failed to wait for debugger run request: {err}"))),
        }
    }

    fn run_gdb_server(mut self, startup_tx: channel::Sender<io::Result<()>>) -> io::Result<()> {
        let sock = TcpListener::bind("127.0.0.1:2159")?;
        println!("GDB server listening on {}", sock.local_addr()?);

        let _ = startup_tx.send(Ok(()));

        loop {
            let (stream, addr) = sock.accept()?;

            println!("GDB client attached from {addr}");

            let gdb = GdbStub::new(stream);

            match gdb.run_blocking::<GdbBlockingEventLoop>(&mut self) {
                Ok(DisconnectReason::Disconnect) => {
                    println!("GDB client requested detach");
                    println!("GDB client detached");
                }
                Ok(DisconnectReason::TargetExited(code)) => {
                    println!("GDB session ended: target exited with code {code}");
                    return Ok(());
                }
                Ok(DisconnectReason::TargetTerminated(sig)) => {
                    println!("GDB session ended: target terminated with signal {sig:?}");
                    return Ok(());
                }
                Ok(DisconnectReason::Kill) => {
                    println!("GDB session ended: kill requested");
                    return Ok(());
                }
                Err(err) if err.is_connection_error() => {
                    if let Some((conn_err, kind)) = err.into_connection_error() {
                        println!("GDB client disconnected");
                        println!("GDB connection closed ({kind:?}): {conn_err}");
                    }
                }
                Err(err) if err.is_target_error() => {
                    return Err(io::Error::other(format!("GDB target error: {}", err.into_target_error().unwrap())));
                }
                Err(err) => {
                    return Err(io::Error::other(format!("GDB server error: {err}")));
                }
            }
            println!("GDB server waiting for next client");
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
        let thread_ids = self.debug.active_threads();

        for thread_id in thread_ids {
            thread_is_active(Tid::try_from(thread_id).unwrap());
        }

        Ok(())
    }
}

impl MultiThreadResume for GdbTarget {
    fn resume(&mut self) -> Result<(), Self::Error> {
        if self.resume_step {
            self.debug.resume_step();
        } else {
            self.debug.resume_continue();
        }

        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        self.resume_step = false;
        Ok(())
    }

    fn set_resume_action_continue(&mut self, _tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume_step = false;
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<MultiThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl MultiThreadSingleStep for GdbTarget {
    fn set_resume_action_step(&mut self, _tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume_step = true;
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
        if self.debug.active_threads().is_empty() {
            self.debug.wait_for_thread_ready();
        }

        Ok(Pid::new(1).unwrap())
    }

    fn attach(&mut self, _pid: Pid) -> TargetResult<(), Self> {
        Ok(())
    }

    fn query_if_attached(&mut self, _pid: Pid) -> TargetResult<AttachKind, Self> {
        Ok(AttachKind::Attach)
    }

    fn kill(&mut self, _pid: Option<Pid>) -> TargetResult<ShouldTerminate, Self> {
        Ok(ShouldTerminate::No)
    }

    fn restart(&mut self) -> Result<(), Self::Error> {
        Ok(())
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

struct GdbBlockingEventLoop;

impl BlockingEventLoop for GdbBlockingEventLoop {
    type Target = GdbTarget;
    type Connection = TcpStream;

    type StopReason = MultiThreadStopReason<u32>;

    fn wait_for_stop_reason(
        target: &mut GdbTarget,
        conn: &mut Self::Connection,
    ) -> Result<Event<MultiThreadStopReason<u32>>, WaitForStopReasonError<GdbTargetError, io::Error>> {
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
