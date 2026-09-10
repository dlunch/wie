use alloc::{boxed::Box, format, rc::Rc, string::String, vec::Vec};
use core::{cell::RefCell, future};

use futures::channel::oneshot;
use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{AbortController, AbortSignal};
use wie_arm_jit_types::{
    AccessResult, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess,
    PreparationFuture, RunFrame,
};

use crate::{Compiler, WasmArtifact};

#[wasm_bindgen(inline_js = r#"
import { compileArm } from "@ts/arm-compiler.ts";
export { compileArm };
export function compilerTask() {
    return new Promise(resolve => setTimeout(resolve, 0));
}
export function executeRegion(region, frame, context) {
    const exit = region(frame, context);
    // Reject non-numbers before the Wasm import can coerce them to valid exits.
    return typeof exit === "number" ? exit : NaN;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = compileArm)]
    fn compile_arm(bytes: &Uint8Array, imports: &Object, frame: u32, names: &Array, deadline: f64, signal: &AbortSignal) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_name = compilerTask)]
    fn compiler_task() -> Promise;

    #[wasm_bindgen(catch, js_name = executeRegion)]
    fn execute_region(region: &Function, frame: u32, context: u32) -> Result<f64, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    pub fn now() -> f64;
}

#[derive(Default)]
struct State {
    closed: bool,
    functions: Vec<Option<Function>>,
    abort: Option<AbortController>,
}

#[derive(Default)]
pub struct WasmExecutor {
    state: Rc<RefCell<State>>,
}

// The browser host is single-agent: JS handles and synchronous borrowed execution
// contexts never leave its main thread.
unsafe impl Send for WasmExecutor {}

impl CompiledExecutor for WasmExecutor {
    fn prepare(&mut self, request: CompileRequest, deadline_ms: f64) -> PreparationFuture {
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Box::pin(future::ready(Err(String::from("compiled executor is closed"))));
        }
        let abort = match AbortController::new() {
            Ok(abort) => abort,
            Err(error) => {
                return Box::pin(future::ready(Err(format!("compiler cancellation setup: {error:?}"))));
            }
        };
        let signal = abort.signal();
        state.abort = Some(abort);
        let weak = Rc::downgrade(&self.state);
        drop(state);
        let (sender, receiver) = oneshot::channel();
        spawn_local(async move {
            let result = prepare_module(request, deadline_ms, &signal).await;
            let Some(state) = weak.upgrade() else { return };
            let mut state = state.borrow_mut();
            if state.closed || signal.aborted() {
                return;
            }
            state.abort = None;
            let result = match result {
                Ok((artifact, functions)) if now() < deadline_ms => {
                    state.functions = functions;
                    Ok(artifact)
                }
                Ok(_) => Err(String::from("ARM AOT preparation timed out")),
                Err(error) => Err(format!("ARM AOT preparation: {error:?}")),
            };
            drop(state);
            let _ = sender.send(result);
        });
        // Only the Send receiver crosses the host initialization future's await.
        Box::pin(async move { receiver.await.map_err(|_| String::from("ARM AOT preparation cancelled"))? })
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String> {
        let state = self.state.borrow();
        let function = state
            .functions
            .get(handle.slot as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| String::from("compiled handle is retired or executor is closed"))?;
        let mut context = ExecutionContext { access, frame };
        let result = execute_region(function, context.frame as u32, &mut context as *mut ExecutionContext<'_> as u32);
        drop(state);
        let exit = match result {
            Ok(value) => match value {
                0.0 => Ok(CompiledExit::Dispatch),
                1.0 => Ok(CompiledExit::Sample),
                2.0 => Ok(CompiledExit::Budget),
                3.0 => Ok(CompiledExit::End),
                4.0 => Ok(CompiledExit::InterpretOne),
                6.0 => Ok(CompiledExit::GuestFault),
                _ => Err(format!("compiled ABI returned invalid exit: {value:?}")),
            },
            Err(error) => Err(format!("generated code trapped: {error:?}")),
        };
        if exit.is_err() {
            self.shutdown();
        }
        exit
    }

    fn retire(&mut self, handles: &[CompiledHandle]) {
        let mut state = self.state.borrow_mut();
        for handle in handles {
            if let Some(function) = state.functions.get_mut(handle.slot as usize) {
                *function = None;
            }
        }
    }

    fn shutdown(&mut self) {
        let abort = {
            let mut state = self.state.borrow_mut();
            state.closed = true;
            state.functions.clear();
            state.abort.take()
        };
        if let Some(abort) = abort {
            abort.abort();
        }
    }
}

impl Drop for WasmExecutor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn prepare_module(request: CompileRequest, deadline: f64, signal: &AbortSignal) -> Result<(CompiledArtifact, Vec<Option<Function>>), JsValue> {
    let started = now();
    let memory_before = core::arch::wasm32::memory_size::<0>() * 65536;
    let mut compiler_ms = None;
    let mut outputcopy_ms = None;
    let mut encoded_size = 0;
    let mut region_count = 0;
    let result = async {
        let mut compiler = Compiler::new(request);
        let mut group_started = now();
        loop {
            if signal.aborted() || now() >= deadline {
                return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
            }
            let complete = compiler.step().map_err(|error| JsValue::from_str(&error))?;
            if now() - group_started >= 4.0 {
                JsFuture::from(compiler_task()).await?;
                group_started = now();
            }
            if complete {
                break;
            }
        }
        let WasmArtifact { bytes, manifest } = compiler.finish();
        compiler_ms = Some(now() - started);
        let output_started = now();
        encoded_size = bytes.len();
        region_count = manifest.len();
        let output = Uint8Array::new_with_length(encoded_size as u32);
        for (index, chunk) in bytes.chunks(64 * 1024).enumerate() {
            if signal.aborted() || now() >= deadline {
                return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
            }
            let offset = index * 64 * 1024;
            output.subarray(offset as u32, (offset + chunk.len()) as u32).copy_from(chunk);
            if now() - group_started >= 4.0 {
                JsFuture::from(compiler_task()).await?;
                group_started = now();
            }
        }
        drop(bytes);
        let names = Array::new();
        for region in &manifest {
            if signal.aborted() || now() >= deadline {
                return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
            }
            names.push(&region.export.as_str().into());
            if now() - group_started >= 4.0 {
                JsFuture::from(compiler_task()).await?;
                group_started = now();
            }
        }
        outputcopy_ms = Some(now() - output_started);
        if signal.aborted() || now() >= deadline {
            return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
        }
        let mut warmup = Box::new(RunFrame {
            cpsr: 0x1f,
            sample_remaining: 1,
            ..RunFrame::default()
        });
        warmup.regs[15] = 0x1000;
        let imports = execution_imports()?;
        let promise = compile_arm(&output, &imports, &mut *warmup as *mut RunFrame as u32, &names, deadline, signal)?;
        drop(output);
        drop(names);
        let result = JsFuture::from(promise).await;
        // The allocation must remain live until the TS job has stopped calling its exports.
        drop(warmup);
        let exports = result?.dyn_into::<Array>()?;
        let mut regions = Vec::with_capacity(manifest.len());
        let mut functions = Vec::with_capacity(manifest.len());
        group_started = now();
        for (index, manifest) in manifest.into_iter().enumerate() {
            if signal.aborted() || now() >= deadline {
                return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
            }
            functions.push(Some(exports.get(index as u32).dyn_into::<Function>()?));
            regions.push(CompiledRegion {
                manifest,
                handle: CompiledHandle { slot: index as u32 },
            });
            exports.set(index as u32, JsValue::UNDEFINED);
            if now() - group_started >= 4.0 {
                JsFuture::from(compiler_task()).await?;
                group_started = now();
            }
        }
        if signal.aborted() || now() >= deadline {
            return Err(JsValue::from_str("ARM AOT preparation cancelled or timed out"));
        }
        Ok((CompiledArtifact { regions, encoded_size }, functions))
    }
    .await;
    let elapsed_ms = now() - started;
    let compiler_ms = compiler_ms.unwrap_or(elapsed_ms);
    let outputcopy_ms = outputcopy_ms.unwrap_or(elapsed_ms - compiler_ms);
    let memory_retained = core::arch::wasm32::memory_size::<0>() * 65536;
    tracing::info!(
        compiler_ms,
        outputcopy_ms,
        elapsed_ms,
        encoded_size,
        region_count,
        hostMemoryBefore = memory_before,
        hostMemoryPeak = memory_retained,
        hostMemoryRetained = memory_retained,
        outcome = if result.is_ok() { "ready" } else { "failed" },
        "ARM AOT compiled"
    );
    result
}

fn execution_imports() -> Result<Object, JsValue> {
    let exports = wasm_bindgen::exports();
    let wie = Object::new();
    Reflect::set(&wie, &"memory".into(), &wasm_bindgen::memory())?;
    for (import, export) in [
        ("load", "wie_aot_load"),
        ("store", "wie_aot_store"),
        ("sample_prepare", "wie_aot_sample_prepare"),
        ("word_range", "wie_aot_word_range"),
    ] {
        Reflect::set(&wie, &import.into(), &Reflect::get(&exports, &export.into())?)?;
    }
    let imports = Object::new();
    Reflect::set(&imports, &"wie".into(), &wie)?;
    Ok(imports)
}

struct ExecutionContext<'a> {
    access: &'a mut dyn ExecutionAccess,
    frame: *mut RunFrame,
}

// Only generated code calls these raw exports, synchronously within execute().
// The thin pointer addresses a borrowed context, never the trait object's data.
#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_load(access: u32, address: u32, width: u32, out: u32) -> u32 {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    match context.access.load(address, width) {
        AccessResult::Complete(value) => {
            unsafe { *(out as *mut u32) = value };
            0
        }
        AccessResult::InterpretOne => 1,
        AccessResult::Fault(address) => {
            unsafe { (*context.frame).fault_address = address };
            2
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_store(access: u32, address: u32, width: u32, value: u32) -> u32 {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    match context.access.store(address, width, value) {
        AccessResult::Complete(_) => 0,
        AccessResult::InterpretOne => 1,
        AccessResult::Fault(address) => {
            unsafe { (*context.frame).fault_address = address };
            2
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_sample_prepare(access: u32, pc: u32, r7: u32) {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    context.access.sample_prepare(pc, r7);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_word_range(access: u32, address: u32, words: u32) -> u64 {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    let Some((first, second)) = context.access.word_range(address, words) else {
        return 0;
    };
    // ExecutionAccess is a safe, public trait: validate its spans before exposing unchecked Wasm accesses.
    if first.is_empty() || !first.len().is_multiple_of(4) || first.len().checked_add(second.len()) != Some(words as usize * 4) {
        return 0;
    }
    // Generated code consumes these borrowed spans before calling another helper.
    unsafe { (*context.frame).scratch = first.len() as u32 };
    u64::from(first.as_mut_ptr() as u32) | (u64::from(second.as_mut_ptr() as u32) << 32)
}
