extern crate std; // we need thread

use std::{
    io,
    net::{TcpListener, TcpStream},
    println,
    sync::mpsc,
    thread,
    time::Duration,
};

use gdbstub::{
    common::{Signal, Tid},
    conn::ConnectionExt,
    stub::{
        GdbStub, MultiThreadStopReason,
        run_blocking::{BlockingEventLoop, Event, WaitForStopReasonError},
    },
    target::{
        Target, TargetResult,
        ext::base::{
            BaseOps,
            multithread::MultiThreadBase,
            single_register_access::{SingleRegisterAccess, SingleRegisterAccessOps},
        },
    },
};
use gdbstub_arch::arm::{
    Armv4t,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};

use wie_util::{ByteRead, ByteWrite};

use crate::{ArmCore, context::ArmCoreContext};

type GdbTargetError = &'static str;

#[allow(dead_code)]
pub struct GdbTarget {
    core: ArmCore,
    stop_event_tx: mpsc::Sender<MultiThreadStopReason<u32>>,
    stop_event_rx: mpsc::Receiver<MultiThreadStopReason<u32>>,
}

impl GdbTarget {
    pub fn start(core: ArmCore) {
        let (tx, rx) = mpsc::channel();
        let this = GdbTarget {
            core,
            stop_event_tx: tx,
            stop_event_rx: rx,
        };
        thread::spawn(|| this.run_gdb_server());
    }

    fn run_gdb_server(mut self) -> io::Result<()> {
        let sock = TcpListener::bind("127.0.0.1:2159")?;
        println!("GDB server listening on {}", sock.local_addr()?);

        let (stream, _addr) = sock.accept()?;

        let gdb = GdbStub::new(stream);
        gdb.run_blocking::<GdbBlockingEventLoop>(&mut self).unwrap();

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
}

impl MultiThreadBase for GdbTarget {
    #[inline(always)]
    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, Tid, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn read_registers(&mut self, regs: &mut ArmCoreRegs, tid: Tid) -> TargetResult<(), Self> {
        let thread_context = self.core.read_thread_context(tid.into()).unwrap();

        regs.r[0] = thread_context.r0;
        regs.r[1] = thread_context.r1;
        regs.r[2] = thread_context.r2;
        regs.r[3] = thread_context.r3;
        regs.r[4] = thread_context.r4;
        regs.r[5] = thread_context.r5;
        regs.r[6] = thread_context.r6;
        regs.r[7] = thread_context.r7;
        regs.r[8] = thread_context.r8;
        regs.r[9] = thread_context.sb;
        regs.r[10] = thread_context.sl;
        regs.r[11] = thread_context.fp;
        regs.r[12] = thread_context.ip;
        regs.sp = thread_context.sp;
        regs.lr = thread_context.lr;
        regs.pc = thread_context.pc;
        regs.cpsr = thread_context.cpsr;

        Ok(())
    }

    #[inline(always)]
    fn write_registers(&mut self, regs: &ArmCoreRegs, tid: Tid) -> TargetResult<(), Self> {
        let context = ArmCoreContext {
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
        };

        self.core.write_thread_context(tid.into(), &context);

        Ok(())
    }

    #[inline(always)]
    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8], _tid: Tid) -> TargetResult<usize, Self> {
        Ok(self.core.read_bytes(start_addr, data).unwrap())
    }

    #[inline(always)]
    fn write_addrs(&mut self, start_addr: u32, data: &[u8], _tid: Tid) -> TargetResult<(), Self> {
        self.core.write_bytes(start_addr, data).unwrap();

        Ok(())
    }

    #[inline(always)]
    fn list_active_threads(&mut self, thread_is_active: &mut dyn FnMut(Tid)) -> Result<(), Self::Error> {
        let thread_ids = self.core.get_thread_ids();

        for thread_id in thread_ids {
            thread_is_active(Tid::try_from(thread_id).unwrap());
        }

        Ok(())
    }
}

impl SingleRegisterAccess<Tid> for GdbTarget {
    fn read_register(&mut self, _tid: Tid, _reg_id: ArmCoreRegId, _buf: &mut [u8]) -> TargetResult<usize, Self> {
        todo!()
    }

    fn write_register(&mut self, _tid: Tid, _reg_id: ArmCoreRegId, _val: &[u8]) -> TargetResult<(), Self> {
        todo!()
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
            if let Ok(x) = target.stop_event_rx.try_recv() {
                return Ok(Event::TargetStopped(x));
            }
            if let Some(x) = conn.peek().unwrap() {
                return Ok(Event::IncomingData(x));
            }

            thread::sleep(Duration::from_millis(10)); // TODO is there a better way to do this?
        }
    }

    fn on_interrupt(_target: &mut GdbTarget) -> Result<Option<MultiThreadStopReason<u32>>, GdbTargetError> {
        // TODO handle interrupt

        Ok(Some(MultiThreadStopReason::Signal(Signal::SIGINT)))
    }
}
