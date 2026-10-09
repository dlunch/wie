use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::{AtomicU32, Ordering};

use wie_arm_jit_types::{CompileRequest, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, PreparationFuture, RunFrame};
use wie_core_arm_native::NativeExecutor;
use wie_util::Result;

pub struct TestNativeExecutor {
    inner: NativeExecutor,
    pub retired: Arc<AtomicU32>,
}

impl Default for TestNativeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TestNativeExecutor {
    pub fn new() -> Self {
        Self {
            inner: NativeExecutor::new(),
            retired: Arc::new(AtomicU32::new(0)),
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
        let result = self.inner.execute(handle, frame, access);
        self.retired.fetch_add(frame.executed, Ordering::Relaxed);
        result
    }
}
