#![no_std]
extern crate alloc;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use wasm_bindgen::prelude::*;
use wie_arm_jit::{AccessResult, Admission, CompiledExecutor, CompiledHandle, ExecutionAccess, RunFrame};
use wie_arm_wasm::WasmExecutor;

#[global_allocator]
static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[wasm_bindgen]
pub struct Probe {
    executor: WasmExecutor,
    handles: BTreeMap<u64, Vec<CompiledHandle>>,
    #[wasm_bindgen(readonly)]
    pub stores: u32,
}

#[wasm_bindgen]
impl Probe {
    #[wasm_bindgen(constructor)]
    pub fn new(session: u64) -> Result<Self, JsValue> {
        Ok(Self {
            executor: WasmExecutor::new(session).map_err(JsValue::from)?,
            handles: BTreeMap::new(),
            stores: 0,
        })
    }

    pub fn submit(&mut self, payload: &str) -> Result<u32, JsValue> {
        let request = serde_json::from_str(payload).map_err(|_| JsValue::from_str("invalid test request"))?;
        match self.executor.submit(request) {
            Admission::Accepted => Ok(0),
            Admission::Busy => Ok(1),
            Admission::Failed(error) => Err(error.into()),
        }
    }

    pub fn poll(&mut self) -> String {
        let Some(completion) = self.executor.poll() else { return String::new() };
        match completion.result {
            Ok(artifact) => {
                self.handles
                    .insert(completion.request, artifact.regions.iter().map(|region| region.handle).collect());
                serde_json::json!({ "request": completion.request, "regions": artifact.regions.len() }).to_string()
            }
            Err(error) => serde_json::json!({ "request": completion.request, "error": error }).to_string(),
        }
    }

    pub fn execute(&mut self, request: u64, end: u32, budget: u32, sample: u32) -> Result<String, JsValue> {
        let mut frame = RunFrame {
            cpsr: 0x30,
            end,
            budget_remaining: budget,
            sample_remaining: sample,
            ..RunFrame::default()
        };
        frame.regs[15] = 0x1000;
        frame.regs[7] = 77;
        let mut access = Access {
            samples: Vec::new(),
            stores: &mut self.stores,
        };
        let exit = self
            .executor
            .execute(self.handles[&request][0], &mut frame, &mut access)
            .map_err(JsValue::from)?;
        Ok(serde_json::json!({
            "exit": exit as u32, "r0": frame.regs[0], "pc": frame.regs[15], "executed": frame.executed,
            "budget": frame.budget_remaining, "sample": frame.sample_remaining, "samples": access.samples,
        })
        .to_string())
    }

    pub fn retire(&mut self, request: u64) {
        if let Some(handles) = self.handles.remove(&request) {
            self.executor.retire(&handles);
        }
    }

    pub fn shutdown(&mut self) {
        self.executor.shutdown();
        self.handles.clear();
    }
}

struct Access<'a> {
    samples: Vec<[u32; 3]>,
    stores: &'a mut u32,
}

impl ExecutionAccess for Access<'_> {
    fn load(&mut self, _address: u32, _width: u32) -> AccessResult {
        AccessResult::InterpretOne
    }

    fn store(&mut self, _address: u32, _width: u32, _value: u32) -> AccessResult {
        *self.stores += 1;
        AccessResult::Complete(0)
    }

    fn sample_prepare(&mut self, pc: u32, cpsr: u32, r7: u32) {
        self.samples.push([pc, cpsr, r7]);
    }
}
