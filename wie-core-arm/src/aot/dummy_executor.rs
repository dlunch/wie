use alloc::{boxed::Box, string::String, vec::Vec};

use wie_arm_jit_types::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, PreparationFuture, RunFrame,
};

pub(crate) struct DummyExecutor;

impl CompiledExecutor for DummyExecutor {
    fn prepare(&mut self, request: CompileRequest, _deadline_ms: f64) -> PreparationFuture {
        Box::pin(async move {
            for _region in request.regions {}
            Ok(CompiledArtifact {
                regions: Vec::new(),
                encoded_size: 0,
            })
        })
    }

    fn execute(&mut self, _handle: CompiledHandle, _frame: &mut RunFrame, _access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String> {
        Ok(CompiledExit::InterpretOne)
    }

    fn retire(&mut self, _handles: &[CompiledHandle]) {}
}
