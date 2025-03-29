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
        run_blocking::{self, BlockingEventLoop},
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

use crate::ArmCore;

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
    fn read_registers(&mut self, _regs: &mut ArmCoreRegs, _tid: Tid) -> TargetResult<(), Self> {
        todo!()
    }

    #[inline(always)]
    fn write_registers(&mut self, _regs: &ArmCoreRegs, _tid: Tid) -> TargetResult<(), Self> {
        todo!()
    }

    #[inline(always)]
    fn read_addrs(&mut self, _start_addr: u32, _data: &mut [u8], _tid: Tid) -> TargetResult<usize, Self> {
        todo!()
    }

    #[inline(always)]
    fn write_addrs(&mut self, _start_addr: u32, _data: &[u8], _tid: Tid) -> TargetResult<(), Self> {
        todo!()
    }

    #[inline(always)]
    fn list_active_threads(&mut self, _thread_is_active: &mut dyn FnMut(Tid)) -> Result<(), Self::Error> {
        todo!()
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
    ) -> Result<run_blocking::Event<MultiThreadStopReason<u32>>, run_blocking::WaitForStopReasonError<GdbTargetError, io::Error>> {
        loop {
            if let Ok(x) = target.stop_event_rx.try_recv() {
                return Ok(run_blocking::Event::TargetStopped(x));
            }
            if let Some(x) = conn.peek().unwrap() {
                return Ok(run_blocking::Event::IncomingData(x));
            }

            thread::sleep(Duration::from_millis(10)); // TODO is there a better way to do this?
        }
    }

    fn on_interrupt(_target: &mut GdbTarget) -> Result<Option<MultiThreadStopReason<u32>>, GdbTargetError> {
        // TODO handle interrupt

        Ok(Some(MultiThreadStopReason::Signal(Signal::SIGINT)))
    }
}
