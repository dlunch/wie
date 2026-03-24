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
    common::{Pid, Signal, Tid},
    conn::ConnectionExt,
    stub::{
        DisconnectReason, GdbStub, MultiThreadStopReason,
        run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError},
    },
    target::{
        Target, TargetResult,
        ext::base::{
            BaseOps,
            multithread::{MultiThreadBase, MultiThreadResume, MultiThreadSingleStep, MultiThreadSingleStepOps},
        },
        ext::breakpoints::{Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps},
        ext::extended_mode::{AttachKind, CurrentActivePid, CurrentActivePidOps, ExtendedMode, ExtendedModeOps, ShouldTerminate},
    },
};
use gdbstub_arch::arm::{Armv4t, reg::ArmCoreRegs};

use crate::{ArmCore, engine::DebugInner};

type GdbTargetError = &'static str;

#[allow(dead_code)]
pub struct GdbTarget {
    core: ArmCore,
    debug: Arc<DebugInner>,
    do_step: bool,
    startup_tx: Option<channel::Sender<io::Result<()>>>,
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
            do_step: false,
            startup_tx: Some(startup_tx),
        };

        thread::spawn(|| {
            let _ = this.run_gdb_server();
        });

        match startup_rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(wie_util::WieError::FatalError(format!("Failed to start GDB server: {err}"))),
            Err(err) => Err(wie_util::WieError::FatalError(format!("Failed to wait for debugger run request: {err}"))),
        }
    }

    fn run_gdb_server(mut self) -> io::Result<()> {
        let sock = TcpListener::bind("127.0.0.1:2159")?;
        println!("GDB server listening on {}", sock.local_addr()?);

        loop {
            let (stream, addr) = match sock.accept() {
                Ok(result) => result,
                Err(err) => {
                    if let Some(startup_tx) = self.startup_tx.take() {
                        let _ = startup_tx.send(Err(io::Error::new(err.kind(), format!("{err}"))));
                    }
                    return Err(err);
                }
            };

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

        #[allow(unreachable_code)]
        Ok(())
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
    fn support_resume(&mut self) -> Option<gdbstub::target::ext::base::multithread::MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn read_registers(&mut self, regs: &mut ArmCoreRegs, _tid: Tid) -> TargetResult<(), Self> {
        let thread_context = self.debug.read_registers();

        regs.r[0] = thread_context[0];
        regs.r[1] = thread_context[1];
        regs.r[2] = thread_context[2];
        regs.r[3] = thread_context[3];
        regs.r[4] = thread_context[4];
        regs.r[5] = thread_context[5];
        regs.r[6] = thread_context[6];
        regs.r[7] = thread_context[7];
        regs.r[8] = thread_context[8];
        regs.r[9] = thread_context[9];
        regs.r[10] = thread_context[10];
        regs.r[11] = thread_context[11];
        regs.r[12] = thread_context[12];
        regs.sp = thread_context[13];
        regs.lr = thread_context[14];
        regs.pc = thread_context[15];
        regs.cpsr = thread_context[16];

        Ok(())
    }

    #[inline(always)]
    fn write_registers(&mut self, regs: &ArmCoreRegs, _tid: Tid) -> TargetResult<(), Self> {
        self.debug.write_registers([
            regs.r[0], regs.r[1], regs.r[2], regs.r[3], regs.r[4], regs.r[5], regs.r[6], regs.r[7], regs.r[8], regs.r[9], regs.r[10], regs.r[11],
            regs.r[12], regs.sp, regs.lr, regs.pc, regs.cpsr,
        ]);

        Ok(())
    }

    #[inline(always)]
    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8], _tid: Tid) -> TargetResult<usize, Self> {
        self.debug
            .read_memory(start_addr, data)
            .map_err(|_| gdbstub::target::TargetError::NonFatal)
    }

    #[inline(always)]
    fn write_addrs(&mut self, start_addr: u32, data: &[u8], _tid: Tid) -> TargetResult<(), Self> {
        self.debug
            .write_memory(start_addr, data)
            .map_err(|_| gdbstub::target::TargetError::NonFatal)
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
        if self.do_step {
            self.debug.resume_step();
        } else {
            self.debug.resume_continue();
        }

        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        self.do_step = false;
        Ok(())
    }

    fn set_resume_action_continue(&mut self, _tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.do_step = false;
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<MultiThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl MultiThreadSingleStep for GdbTarget {
    fn set_resume_action_step(&mut self, _tid: Tid, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.do_step = true;
        Ok(())
    }
}

impl Breakpoints for GdbTarget {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for GdbTarget {
    fn add_sw_breakpoint(&mut self, addr: u32, _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        self.debug.add_breakpoint(addr);

        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: u32, _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        self.debug.remove_breakpoint(addr);

        Ok(true)
    }
}

impl ExtendedMode for GdbTarget {
    fn run(&mut self, _filename: Option<&[u8]>, _args: gdbstub::target::ext::extended_mode::Args<'_, '_>) -> TargetResult<Pid, Self> {
        if let Some(startup_tx) = self.startup_tx.take() {
            let _ = startup_tx.send(Ok(()));
        }

        let _ = self.debug.thread_ready_rx.recv();

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
        let stop_event_rx = target.debug.stop_event_rx.clone();

        loop {
            match stop_event_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(x) => return Ok(Event::TargetStopped(x)),
                Err(channel::RecvTimeoutError::Timeout) => {
                    if conn.peek().unwrap().is_some() {
                        let x = conn.read().unwrap();
                        return Ok(Event::IncomingData(x));
                    }
                }
                Err(_) => return Err(WaitForStopReasonError::Target("Target error")),
            }
        }
    }

    fn on_interrupt(target: &mut GdbTarget) -> Result<Option<MultiThreadStopReason<u32>>, GdbTargetError> {
        target.debug.interrupt();
        Ok(None::<MultiThreadStopReason<u32>>)
    }
}
