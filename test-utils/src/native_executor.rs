use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::{AtomicBool, Ordering};

use wie_arm_jit_types::{CompileRequest, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, PreparationFuture, RunFrame};
use wie_core_arm_native::NativeExecutor;
use wie_util::Result;

pub struct TestNativeExecutor {
    inner: NativeExecutor,
    pub executed: Arc<AtomicBool>,
}

impl Default for TestNativeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TestNativeExecutor {
    pub fn new() -> Self {
        Self {
            inner: NativeExecutor::new(None),
            executed: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl CompiledExecutor for TestNativeExecutor {
    fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
        let preparation = self.inner.prepare(request);
        Box::pin(async move {
            let artifact = preparation.await.expect("native preparation must succeed");
            assert!(!artifact.regions.is_empty(), "native preparation must produce compiled regions");
            Ok(artifact)
        })
    }

    fn release(&mut self, handle: CompiledHandle) {
        self.inner.release(handle);
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
        let pc = frame.regs[15];
        let result = self.inner.execute(handle, frame, access);
        if result.is_ok() && frame.regs[15] != pc {
            self.executed.store(true, Ordering::Relaxed);
        }
        result
    }
}
