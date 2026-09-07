extern crate std; // we need thread

use alloc::{format, sync::Arc, vec::Vec};
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
    step_thread: Option<crate::ThreadId>,
    resumed_threads: Vec<crate::ThreadId>,
    scheduler_locked: bool,
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

        let sock = TcpListener::bind("127.0.0.1:2159").map_err(|err| wie_util::WieError::FatalError(format!("Failed to start GDB server: {err}")))?;
        let this = GdbTarget {
            core,
            debug,
            step_thread: None,
            resumed_threads: Vec::new(),
            scheduler_locked: false,
        };

        thread::Builder::new()
            .spawn(move || {
                if let Err(err) = this.run_gdb_server(sock) {
                    tracing::error!("GDB server error: {err}");
                }
            })
            .map_err(|err| wie_util::WieError::FatalError(format!("Failed to start GDB server thread: {err}")))?;
        Ok(())
    }

    fn run_gdb_server(mut self, sock: TcpListener) -> io::Result<()> {
        println!("GDB server listening on {}", sock.local_addr()?);

        loop {
            let (stream, addr) = sock.accept()?;

            println!("GDB client attached from {addr}");

            match self.run_session(stream) {
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
                Err(err) => {
                    tracing::warn!("GDB session ended: {err}");
                }
            }
            println!("GDB server waiting for next client");
        }
    }

    fn run_session(&mut self, stream: TcpStream) -> io::Result<DisconnectReason> {
        self.debug.pause();
        self.clear_resume_actions().map_err(io::Error::other)?;
        let result = GdbStub::new(stream).run_blocking::<GdbBlockingEventLoop>(self);
        self.debug
            .detach()
            .map_err(|err| io::Error::other(format!("Failed to detach GDB: {err}")))?;
        result.map_err(|err| io::Error::other(format!("{err}")))
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
            self.step_thread,
            self.scheduler_locked.then(|| core::mem::take(&mut self.resumed_threads)),
        );

        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        self.step_thread = None;
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
        self.step_thread = Some(tid.get());
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

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, vec::Vec};
    use std::io::{Read, Write};

    use crate::{Allocator, engine::DebuggedArm32CpuEngine};

    use super::*;

    fn send_packet(stream: &mut TcpStream, payload: &str) {
        let checksum = payload.bytes().fold(0u8, u8::wrapping_add);
        write!(stream, "${payload}#{checksum:02x}").unwrap();
    }

    fn read_packet(stream: &mut TcpStream) -> String {
        let mut byte = [0];
        loop {
            Read::read_exact(stream, &mut byte).unwrap();
            if byte[0] == b'$' {
                break;
            }
        }
        let mut payload = Vec::new();
        loop {
            Read::read_exact(stream, &mut byte).unwrap();
            if byte[0] == b'#' {
                break;
            }
            payload.push(byte[0]);
        }
        let mut checksum = [0; 2];
        Read::read_exact(stream, &mut checksum).unwrap();
        assert_eq!(
            u8::from_str_radix(core::str::from_utf8(&checksum).unwrap(), 16).unwrap(),
            payload.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte))
        );
        Write::write_all(stream, b"+").unwrap();
        let mut decoded = Vec::new();
        let mut bytes = payload.into_iter();
        while let Some(byte) = bytes.next() {
            if byte == b'*' {
                let count = bytes.next().unwrap() - 29;
                decoded.extend(core::iter::repeat_n(*decoded.last().unwrap(), count as usize));
            } else {
                decoded.push(byte);
            }
        }
        String::from_utf8(decoded).unwrap()
    }

    #[test]
    fn remote_sessions_read_threads_step_interrupt_and_reattach() {
        let mut core = ArmCore::new(false, None).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        Allocator::init(&mut core).unwrap();
        core.load(&[0x01, 0x30, 0xfd, 0xe7], 0x1000, 4).unwrap(); // add r0, #1; b 0x1000
        let _parked = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let mut context = core.read_thread_context(1).unwrap();
        context.r0 = 42;
        core.write_thread_context(1, &context);

        let mut running_core = core.clone();
        let task = core
            .run_in_thread(move || async move {
                running_core.run_function::<()>(0x1001, &[0]).await?;
                Ok(())
            })
            .unwrap();
        let runner = thread::spawn(move || futures::executor::block_on(task));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut target = GdbTarget {
            core,
            debug: debug.clone(),
            step_thread: None,
            resumed_threads: Vec::new(),
            scheduler_locked: false,
        };
        target.set_resume_action_step(Tid::new(2).unwrap(), None).unwrap();
        target.set_resume_action_continue(Tid::new(1).unwrap(), None).unwrap();
        assert_eq!(target.step_thread, Some(2));
        let server = thread::spawn(move || {
            for session in 0..3 {
                let (stream, _) = listener.accept().unwrap();
                let result = target.run_session(stream);
                if session == 1 {
                    assert!(result.is_err());
                } else {
                    assert!(matches!(result.unwrap(), DisconnectReason::Disconnect));
                }
            }
        });

        for session in 0..3 {
            let mut stream = TcpStream::connect(address).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            send_packet(&mut stream, "qSupported:multiprocess+;swbreak+");
            assert!(read_packet(&mut stream).contains("multiprocess+"));
            send_packet(&mut stream, "?");
            assert!(read_packet(&mut stream).contains("thread:p01.02;"));
            send_packet(&mut stream, "qfThreadInfo");
            assert_eq!(read_packet(&mut stream), "mp01.02,p01.01");

            if session == 0 {
                send_packet(&mut stream, "Hgp1.1");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "g");
                let regs = read_packet(&mut stream);
                assert_eq!(&regs[..8], "2a000000");
                send_packet(&mut stream, "Hgp1.99");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "g");
                assert!(read_packet(&mut stream).starts_with('E'));
                send_packet(&mut stream, "Hgp1.2");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "Z0,1002,2");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "vCont;c");
                assert!(read_packet(&mut stream).contains("swbreak:;"));
                send_packet(&mut stream, "vCont;s:p1.2");
                let stopped = read_packet(&mut stream);
                assert!(stopped.starts_with("T05thread:p01.02;"));
                assert!(!stopped.contains("swbreak"));
                send_packet(&mut stream, "g");
                let regs = read_packet(&mut stream);
                assert_eq!(&regs[..8], "01000000");
                assert_eq!(&regs[15 * 8..16 * 8], "00100000");
            } else if session == 1 {
                send_packet(&mut stream, "Z0,1002,2");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "vCont;c");
                drop(stream);
                thread::sleep(Duration::from_millis(10));
                continue;
            } else {
                assert!(!debug.has_breakpoints());
                assert!(debug.read_registers().r0 > 1);
                send_packet(&mut stream, "!");
                assert_eq!(read_packet(&mut stream), "OK");
                send_packet(&mut stream, "vAttach;1");
                assert!(read_packet(&mut stream).contains("thread:p01.02;"));
                send_packet(&mut stream, "vCont;c");
                Write::write_all(&mut stream, &[3]).unwrap();
                assert!(read_packet(&mut stream).starts_with("T02thread:p01.02;"));
                send_packet(&mut stream, "M1000,4:70477047"); // bx lr at either PC in the loop
                assert_eq!(read_packet(&mut stream), "OK");
            }
            send_packet(&mut stream, "D;1");
            assert_eq!(read_packet(&mut stream), "OK");
            drop(stream);
            if session == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        server.join().unwrap();
        runner.join().unwrap().unwrap();
    }
}
