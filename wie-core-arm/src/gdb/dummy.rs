use alloc::format;

use gdbstub::{
    conn::{Connection, ConnectionExt},
    stub::GdbStub,
};

use crate::ArmCore;

use super::{GdbBlockingEventLoop, GdbTarget};

const UNAVAILABLE: &str = "GDB dummy transport is unavailable";

struct DummyConnection;

impl Connection for DummyConnection {
    type Error = &'static str;

    fn write(&mut self, _byte: u8) -> Result<(), Self::Error> {
        Err(UNAVAILABLE)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Err(UNAVAILABLE)
    }

    fn on_session_start(&mut self) -> Result<(), Self::Error> {
        Err(UNAVAILABLE)
    }
}

impl ConnectionExt for DummyConnection {
    fn read(&mut self) -> Result<u8, Self::Error> {
        Err(UNAVAILABLE)
    }

    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        Err(UNAVAILABLE)
    }
}

pub(crate) fn start(core: ArmCore) -> wie_util::Result<()> {
    let mut target = GdbTarget::new(core);
    // Fail during connection initialization, before waiting for a guest thread.
    GdbStub::new(DummyConnection)
        .run_blocking::<GdbBlockingEventLoop<DummyConnection>>(&mut target)
        .map_err(|err| wie_util::WieError::FatalError(format!("{err}")))?;
    target.debug.detach()
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::engine::DebuggedArm32CpuEngine;

    use super::*;

    #[test]
    fn unavailable_transport_fails_before_any_guest_thread_exists() {
        let core = ArmCore::new(false, None).unwrap();
        core.inner.lock().engine = Box::new(DebuggedArm32CpuEngine::new());
        let result = start(core);
        assert!(matches!(result, Err(wie_util::WieError::FatalError(message)) if message.contains(UNAVAILABLE)));
    }
}
