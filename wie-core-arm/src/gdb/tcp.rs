extern crate std;

use alloc::format;
use std::{
    io,
    net::{TcpListener, TcpStream},
    println, thread,
};

use gdbstub::{
    stub::{DisconnectReason, GdbStub},
    target::ext::base::multithread::MultiThreadResume,
};

use crate::ArmCore;

use super::{GdbBlockingEventLoop, GdbTarget};

pub(crate) fn start(core: ArmCore) -> wie_util::Result<()> {
    let sock = TcpListener::bind("127.0.0.1:2159").map_err(|err| wie_util::WieError::FatalError(format!("Failed to start GDB server: {err}")))?;
    let this = GdbTarget::new(core);
    thread::Builder::new()
        .spawn(move || {
            if let Err(err) = this.run_gdb_server(sock) {
                tracing::error!("GDB server error: {err}");
            }
        })
        .map_err(|err| wie_util::WieError::FatalError(format!("Failed to start GDB server thread: {err}")))?;
    Ok(())
}

impl GdbTarget {
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
        let result = GdbStub::new(stream).run_blocking::<GdbBlockingEventLoop<TcpStream>>(self);
        self.debug
            .detach()
            .map_err(|err| io::Error::other(format!("Failed to detach GDB: {err}")))?;
        result.map_err(|err| io::Error::other(format!("{err}")))
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, vec::Vec};
    use core::{
        pin::Pin,
        task::{Context, Poll, Waker},
        time::Duration,
    };
    use std::io::{Read, Write};

    use gdbstub::{common::Tid, target::ext::base::multithread::MultiThreadSingleStep};

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
            step_threads: Vec::new(),
            resumed_threads: Vec::new(),
            scheduler_locked: false,
        };
        target.set_resume_action_step(Tid::new(2).unwrap(), None).unwrap();
        target.set_resume_action_continue(Tid::new(1).unwrap(), None).unwrap();
        assert_eq!(target.step_threads, [2]);
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

    #[test]
    fn remote_step_actions_remain_bound_to_their_threads() {
        let mut core = ArmCore::new(false, None).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        Allocator::init(&mut core).unwrap();
        let mut tasks = Vec::new();
        for address in [0x1000, 0x2000] {
            core.load(&[0x01, 0x30, 0xfd, 0xe7], address, 4).unwrap(); // add r0, #1; loop
            let mut running_core = core.clone();
            tasks.push(
                core.run_in_thread(move || async move { running_core.run_function::<()>(address | 1, &[0]).await })
                    .unwrap(),
            );
        }
        let runner = thread::spawn(move || {
            let mut cx = Context::from_waker(Waker::noop());
            while !tasks.is_empty() {
                tasks.retain_mut(|task| match Pin::new(task).poll(&mut cx) {
                    Poll::Pending => true,
                    Poll::Ready(result) => {
                        result.unwrap();
                        false
                    }
                });
            }
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let mut target = GdbTarget {
                core,
                debug,
                step_threads: Vec::new(),
                resumed_threads: Vec::new(),
                scheduler_locked: false,
            };
            let (stream, _) = listener.accept().unwrap();
            assert!(matches!(target.run_session(stream).unwrap(), DisconnectReason::Disconnect));
        });
        let mut stream = TcpStream::connect(address).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        send_packet(&mut stream, "qSupported:multiprocess+;swbreak+");
        read_packet(&mut stream);
        send_packet(&mut stream, "Z0,1000,2");
        assert_eq!(read_packet(&mut stream), "OK");
        send_packet(&mut stream, "vCont;c");
        assert!(read_packet(&mut stream).contains("swbreak:;"));
        send_packet(&mut stream, "vCont;s:p1.2");
        assert_eq!(read_packet(&mut stream), "T05thread:p01.02;");
        send_packet(&mut stream, "vCont;s:p1.1");
        let stepped_breakpoint = read_packet(&mut stream);

        send_packet(&mut stream, "z0,1000,2");
        assert_eq!(read_packet(&mut stream), "OK");
        send_packet(&mut stream, "vCont;s:p1.1");
        let stepped_branch = read_packet(&mut stream);
        send_packet(&mut stream, "vCont;s:p1.1;s:p1.2");
        let stepped_threads = read_packet(&mut stream);
        let mut registers = Vec::new();
        for thread in [1, 2] {
            send_packet(&mut stream, &format!("Hgp1.{thread}"));
            assert_eq!(read_packet(&mut stream), "OK");
            send_packet(&mut stream, "g");
            registers.push(read_packet(&mut stream));
        }

        for command in ["M1000,4:70477047", "M2000,4:70477047", "D;1"] {
            send_packet(&mut stream, command);
            assert_eq!(read_packet(&mut stream), "OK");
        }
        server.join().unwrap();
        runner.join().unwrap();
        assert_eq!(stepped_breakpoint, "T05thread:p01.01;");
        assert_eq!(stepped_branch, "T05thread:p01.01;");
        // Either resumed thread can reach its step first; all-stop keeps the other unchanged.
        let expected = match stepped_threads.as_str() {
            "T05thread:p01.01;" => [("02000000", "02100000"), ("01000000", "02200000")],
            "T05thread:p01.02;" => [("01000000", "00100000"), ("01000000", "00200000")],
            response => panic!("unexpected step response: {response}"),
        };
        for (registers, (r0, pc)) in registers.iter().zip(expected) {
            assert_eq!(&registers[..8], r0);
            assert_eq!(&registers[15 * 8..16 * 8], pc);
        }
    }
}
