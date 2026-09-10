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
    memory: [u8; 256],
    range_lengths: Option<(usize, usize)>,
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
            memory: [0; 256],
            range_lengths: None,
            stores: 0,
        })
    }

    pub fn submit(&mut self, payload: &str) -> Result<u32, JsValue> {
        let request = serde_json::from_str(payload).map_err(|_| JsValue::from_str("invalid test request"))?;
        match self.executor.submit(&request) {
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
            entry_pc: 0x8000,
            ..RunFrame::default()
        };
        frame.regs[15] = 0x1000;
        frame.regs[7] = 77;
        let mut access = Access {
            samples: Vec::new(),
            stores: &mut self.stores,
            memory: &mut self.memory,
            range_lengths: self.range_lengths,
        };
        let exit = self
            .executor
            .execute(self.handles[&request][0], &mut frame, &mut access)
            .map_err(JsValue::from)?;
        Ok(serde_json::json!({
            "exit": exit as u32, "r0": frame.regs[0], "r7": frame.regs[7], "pc": frame.regs[15], "executed": frame.executed,
            "budget": frame.budget_remaining, "sample": frame.sample_remaining, "scratch": frame.scratch, "entry": frame.entry_pc,
            "samples": access.samples,
        })
        .to_string())
    }

    pub fn retire(&mut self, request: u64) {
        if let Some(handles) = self.handles.remove(&request) {
            self.executor.retire(&handles);
        }
    }

    pub fn set_range_lengths(&mut self, first: usize, second: usize) {
        self.range_lengths = Some((first, second));
    }

    pub fn memory(&self) -> Vec<u8> {
        self.memory.to_vec()
    }

    pub fn shutdown(&mut self) {
        self.executor.shutdown();
        self.handles.clear();
    }
}

struct Access<'a> {
    samples: Vec<[u32; 4]>,
    stores: &'a mut u32,
    memory: &'a mut [u8; 256],
    range_lengths: Option<(usize, usize)>,
}

impl ExecutionAccess for Access<'_> {
    fn word_range(&mut self, address: u32, words: u32) -> Option<(&mut [u8], &mut [u8])> {
        if let Some((first, second)) = self.range_lengths {
            return Some(self.memory.get_mut(..first.checked_add(second)?)?.split_at_mut(first));
        }
        if !address.is_multiple_of(4) {
            return None;
        }
        let start = address as usize;
        let end = start.checked_add(words as usize * 4)?;
        Some((self.memory.get_mut(start..end)?, &mut []))
    }

    fn load(&mut self, address: u32, width: u32) -> AccessResult {
        if !address.is_multiple_of(width) || u64::from(address) + u64::from(width) > self.memory.len() as u64 {
            return AccessResult::InterpretOne;
        }
        let mut value = 0;
        for (index, byte) in self.memory[address as usize..(address + width) as usize].iter().enumerate() {
            value |= u32::from(*byte) << (index * 8);
        }
        AccessResult::Complete(value)
    }

    fn store(&mut self, address: u32, width: u32, value: u32) -> AccessResult {
        if !address.is_multiple_of(width) || u64::from(address) + u64::from(width) > self.memory.len() as u64 {
            return AccessResult::InterpretOne;
        }
        self.memory[address as usize..(address + width) as usize].copy_from_slice(&value.to_le_bytes()[..width as usize]);
        *self.stores += 1;
        AccessResult::Complete(0)
    }

    fn sample_prepare(&mut self, pc: u32, cpsr: u32, r7: u32, entry_pc: u32) {
        self.samples.push([pc, cpsr, r7, entry_pc]);
    }
}
