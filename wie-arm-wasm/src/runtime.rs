use alloc::{
    collections::{BTreeMap, VecDeque},
    format,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::cell::RefCell;

use js_sys::{Array, Function, Object, Reflect, Uint8Array, WebAssembly};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{ErrorEvent, MessageEvent, Worker};
use wie_arm_jit_types::{
    AccessResult, Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion,
    ExecutionAccess, ManifestRegion, RunFrame,
};

const RESERVATION: usize = 512 * 1024;
const LIVE_BYTES: usize = 2 * 1024 * 1024;
const PEAK_BYTES: usize = 4 * 1024 * 1024;
const PENDING_IR_BYTES: usize = 1024 * 1024;
const MAX_QUEUED_REQUESTS: usize = 4;
const MAX_LIVE_MODULES: usize = 16;
const MAX_COEXISTING_MODULES: usize = 20;

#[wasm_bindgen(inline_js = r#"
export function createCompilerWorker() {
    return new Worker(new URL("@ts/arm-compiler-worker.ts", import.meta.url), { type: "module" });
}

export function executeRegion(region, frame, context) {
    const exit = region(frame, context);
    // Reject non-numbers before the Wasm import can coerce them to valid exits.
    return typeof exit === "number" ? exit : NaN;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = createCompilerWorker)]
    fn create_compiler_worker() -> Result<Worker, JsValue>;

    #[wasm_bindgen(catch, js_name = executeRegion)]
    fn execute_region(region: &Function, frame: u32, context: u32) -> Result<f64, JsValue>;
}

struct Pending {
    session: u64,
    request: u64,
    manifest: Vec<ManifestRegion>,
    payload: Vec<u8>,
    request_cost: usize,
}

struct Installed {
    functions: Vec<Option<Function>>,
    encoded_size: usize,
}

enum Stage {
    Compiling,
    Installing,
    Ready(Result<Installed, String>),
}

struct Active {
    pending: Pending,
    stage: Stage,
}

#[derive(Default)]
struct State {
    closed: bool,
    ready: bool,
    worker_failure: Option<String>,
    queue: VecDeque<Pending>,
    active: Option<Active>,
    modules: BTreeMap<u64, Installed>,
}

pub struct WasmExecutor {
    session: u64,
    worker: Option<Worker>,
    state: Rc<RefCell<State>>,
    message: Option<Closure<dyn FnMut(MessageEvent)>>,
    error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    message_error: Option<Closure<dyn FnMut(MessageEvent)>>,
}

// The existing browser host is single-agent: none of these JS handles or borrowed
// execution contexts are sent to the compiler worker or another Rust thread.
unsafe impl Send for WasmExecutor {}

impl WasmExecutor {
    pub fn new(session: u64) -> Result<Self, String> {
        let mut executor = Self {
            session,
            worker: None,
            state: Rc::new(RefCell::new(State::default())),
            message: None,
            error: None,
            message_error: None,
        };
        let worker = match create_compiler_worker() {
            Ok(worker) => worker,
            Err(error) => {
                executor.state.borrow_mut().worker_failure = Some(format!("compiler worker creation: {error:?}"));
                return Ok(executor);
            }
        };
        let state = &executor.state;
        let weak = Rc::downgrade(state);
        let message = Closure::new(move |event: MessageEvent| {
            let Some(state) = weak.upgrade() else { return };
            let mut state = state.borrow_mut();
            if state.closed {
                return;
            }
            let data = event.data();
            if let Some(error) = Reflect::get(&data, &"initializationError".into())
                .ok()
                .and_then(|value| value.as_string())
            {
                state.worker_failure = Some(format!("compiler initialization: {error}"));
                return;
            }
            if Reflect::get(&data, &"ready".into()).ok().and_then(|value| value.as_bool()) == Some(true) {
                state.ready = true;
                return;
            }
            let Some(active) = state.active.as_mut() else { return };
            if !matches!(active.stage, Stage::Compiling) {
                return;
            }
            let id = Reflect::get(&data, &"request".into()).ok().and_then(|value| value.as_string());
            if id.as_deref() != Some(active.pending.request.to_string().as_str()) {
                return;
            }
            let decoded = decode_response(&data, &active.pending.manifest);
            let (module, encoded_size) = match decoded {
                Ok(result) => result,
                Err(error) => {
                    active.stage = Stage::Ready(Err(error));
                    return;
                }
            };
            let imports = match execution_imports() {
                Ok(imports) => imports,
                Err(error) => {
                    active.stage = Stage::Ready(Err(format!("compiled imports: {error:?}")));
                    return;
                }
            };
            let request = active.pending.request;
            let count = active.pending.manifest.len();
            active.stage = Stage::Installing;
            let promise = WebAssembly::instantiate_module(&module, &imports);
            let weak = weak.clone();
            // JsFuture owns its uncancellable Promise callbacks. The task must not
            // retain backend state, requests, or cached functions across this await.
            spawn_local(async move {
                let result = JsFuture::from(promise).await;
                let Some(state) = weak.upgrade() else { return };
                let mut state = state.borrow_mut();
                if state.closed {
                    return;
                }
                let Some(active) = state.active.as_mut() else { return };
                if active.pending.request != request || !matches!(active.stage, Stage::Installing) {
                    return;
                }
                active.stage = Stage::Ready((|| {
                    let instance: WebAssembly::Instance = result
                        .map_err(|error| format!("compiled instantiation: {error:?}"))?
                        .dyn_into()
                        .map_err(|_| String::from("compiled instantiation did not return an instance"))?;
                    let exports = instance.exports();
                    let mut functions = Vec::with_capacity(count);
                    for index in 0..count {
                        let name = format!("region_{index}");
                        let function = Reflect::get(&exports, &name.clone().into())
                            .map_err(|error| format!("compiled export {name}: {error:?}"))?
                            .dyn_into::<Function>()
                            .map_err(|_| format!("missing compiled export {name}"))?;
                        functions.push(Some(function));
                    }
                    Ok(Installed { functions, encoded_size })
                })());
            });
        });
        worker.set_onmessage(Some(message.as_ref().unchecked_ref()));
        let weak = Rc::downgrade(state);
        let error = Closure::new(move |event: ErrorEvent| {
            if let Some(state) = weak.upgrade() {
                state.borrow_mut().worker_failure = Some(format!("compiler worker: {}", event.message()));
            }
        });
        worker.set_onerror(Some(error.as_ref().unchecked_ref()));
        let weak = Rc::downgrade(state);
        let message_error = Closure::new(move |_: MessageEvent| {
            if let Some(state) = weak.upgrade() {
                state.borrow_mut().worker_failure = Some(String::from("compiler worker message could not be deserialized"));
            }
        });
        worker.set_onmessageerror(Some(message_error.as_ref().unchecked_ref()));
        executor.worker = Some(worker);
        executor.message = Some(message);
        executor.error = Some(error);
        executor.message_error = Some(message_error);
        Ok(executor)
    }

    fn start_next(&self) {
        let Some(worker) = &self.worker else { return };
        let mut state = self.state.borrow_mut();
        if state.closed || state.active.is_some() || (!state.ready && state.worker_failure.is_none()) {
            return;
        }
        let Some(mut pending) = state.queue.pop_front() else { return };
        let stage = if let Some(error) = &state.worker_failure {
            Stage::Ready(Err(error.clone()))
        } else {
            let message = Object::new();
            let sent = (|| {
                // Copy into a dedicated JS buffer; never transfer the host Wasm memory.
                let payload = Uint8Array::from(core::mem::take(&mut pending.payload).as_slice());
                let transfer = Array::new();
                transfer.push(&payload.buffer());
                Reflect::set(&message, &"request".into(), &pending.request.to_string().into())?;
                Reflect::set(&message, &"payload".into(), &payload)?;
                worker.post_message_with_transfer(&message, &transfer)
            })();
            match sent {
                Ok(()) => Stage::Compiling,
                Err(error) => Stage::Ready(Err(format!("compiler submission: {error:?}"))),
            }
        };
        state.active = Some(Active { pending, stage });
    }
}

impl CompiledExecutor for WasmExecutor {
    fn submit(&mut self, request: &CompileRequest) -> Admission {
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Admission::Failed(String::from("compiled executor is closed"));
        }
        if let Some(error) = &state.worker_failure {
            return Admission::Failed(error.clone());
        }
        if request.session != self.session {
            return Admission::Failed(String::from("compile request has a stale session"));
        }
        let payload = match serde_json::to_vec(request) {
            Ok(payload) => payload,
            Err(error) => return Admission::Failed(error.to_string()),
        };
        // Keep the original conservative reservation until completion is consumed.
        let request_cost = 4 * request.ir_size() + 5 * payload.len();
        if request_cost > PENDING_IR_BYTES {
            return Admission::Failed(String::from("compile request exceeds the IR budget"));
        }
        let pending_cost: usize = state
            .queue
            .iter()
            .chain(state.active.as_ref().map(|active| &active.pending))
            .map(|pending| pending.request_cost)
            .sum();
        if pending_cost + request_cost > PENDING_IR_BYTES {
            return Admission::Busy;
        }
        let pending = state.queue.len() + usize::from(state.active.is_some());
        let bytes: usize = state.modules.values().map(|module| module.encoded_size).sum();
        if state.queue.len() >= MAX_QUEUED_REQUESTS
            || state.modules.len() + pending + 1 > MAX_COEXISTING_MODULES
            || bytes + (pending + 1) * RESERVATION > PEAK_BYTES
        {
            return Admission::Busy;
        }
        let manifest: Vec<_> = request
            .regions
            .iter()
            .enumerate()
            .map(|(index, region)| ManifestRegion {
                entry: region.ir.entry,
                source: region.source.clone(),
                export: format!("region_{index}"),
                expected_old: region.expected_old,
            })
            .collect();
        if !fits_live(&state, &manifest, RESERVATION) {
            return Admission::Busy;
        }
        state.queue.push_back(Pending {
            session: request.session,
            request: request.request,
            manifest,
            payload,
            request_cost,
        });
        drop(state);
        self.start_next();
        Admission::Accepted
    }

    fn poll(&mut self) -> Option<CompileCompletion> {
        self.start_next();
        let mut state = self.state.borrow_mut();
        if let Some(error) = state.worker_failure.clone()
            && let Some(active) = state.active.as_mut()
        {
            // An in-progress main-agent installation must settle before its
            // reservation can be reused, even when the worker has failed.
            if matches!(active.stage, Stage::Compiling) {
                active.stage = Stage::Ready(Err(error));
            }
        }
        let active = state.active.take()?;
        let Stage::Ready(result) = active.stage else {
            state.active = Some(active);
            return None;
        };
        let Pending {
            session, request, manifest, ..
        } = active.pending;
        let result = result.and_then(|installed| {
            if !fits_live(&state, &manifest, installed.encoded_size) {
                return Err(String::from("compiled installation exceeds the live module budget"));
            }
            let encoded_size = installed.encoded_size;
            let regions = manifest
                .into_iter()
                .enumerate()
                .map(|(index, manifest)| CompiledRegion {
                    manifest,
                    handle: CompiledHandle {
                        slot: index as u32,
                        generation: request,
                    },
                })
                .collect();
            state.modules.insert(request, installed);
            Ok(CompiledArtifact { regions, encoded_size })
        });
        let completion = CompileCompletion { session, request, result };
        drop(state);
        self.start_next();
        Some(completion)
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String> {
        let state = self.state.borrow();
        let function = state
            .modules
            .get(&handle.generation)
            .and_then(|module| module.functions.get(handle.slot as usize))
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
            if let Some(module) = state.modules.get_mut(&handle.generation)
                && let Some(function) = module.functions.get_mut(handle.slot as usize)
            {
                *function = None;
            }
        }
        state.modules.retain(|_, module| module.functions.iter().any(Option::is_some));
    }

    fn shutdown(&mut self) {
        let mut state = self.state.borrow_mut();
        state.closed = true;
        state.queue.clear();
        state.active = None;
        state.modules.clear();
        if let Some(worker) = self.worker.take() {
            worker.set_onmessage(None);
            worker.set_onerror(None);
            worker.set_onmessageerror(None);
            worker.terminate();
        }
        self.message = None;
        self.error = None;
        self.message_error = None;
    }
}

impl Drop for WasmExecutor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn fits_live(state: &State, manifest: &[ManifestRegion], output_size: usize) -> bool {
    let mut count = 1;
    let mut bytes = output_size;
    for (&generation, module) in &state.modules {
        let replaced = module.functions.iter().enumerate().all(|(slot, function)| {
            function.is_none()
                || manifest.iter().any(|region| {
                    region.expected_old
                        == Some(CompiledHandle {
                            slot: slot as u32,
                            generation,
                        })
                })
        });
        if !replaced {
            count += 1;
            bytes += module.encoded_size;
        }
    }
    count <= MAX_LIVE_MODULES && bytes <= LIVE_BYTES
}

fn decode_response(data: &JsValue, expected: &[ManifestRegion]) -> Result<(WebAssembly::Module, usize), String> {
    let get = |key: &str| Reflect::get(data, &key.into()).map_err(|error| format!("compiler response: {error:?}"));
    if let Some(error) = get("error")?.as_string() {
        return Err(error);
    }
    let size = get("encodedSize")?
        .as_f64()
        .ok_or_else(|| String::from("compiler response is missing its size"))?;
    if !(1.0..=RESERVATION as f64).contains(&size) || size != (size as usize) as f64 {
        return Err(String::from("compiler response exceeds its output reservation"));
    }
    let manifest = get("manifest")?
        .as_string()
        .ok_or_else(|| String::from("compiler response is missing its manifest"))?;
    let manifest: Vec<ManifestRegion> = serde_json::from_str(&manifest).map_err(|error| error.to_string())?;
    if manifest.len() != expected.len()
        || manifest.iter().zip(expected).any(|(actual, expected)| {
            actual.entry != expected.entry
                || actual.source != expected.source
                || actual.expected_old != expected.expected_old
                || actual.export != expected.export
        })
    {
        return Err(String::from("compiler manifest does not match its request"));
    }
    let module = get("module")?
        .dyn_into()
        .map_err(|_| String::from("compiler response is missing its module"))?;
    Ok((module, size as usize))
}

fn execution_imports() -> Result<Object, JsValue> {
    let exports = wasm_bindgen::exports();
    let wie = Object::new();
    Reflect::set(&wie, &"memory".into(), &wasm_bindgen::memory())?;
    for (import, export) in [
        ("load", "wie_jit_load"),
        ("store", "wie_jit_store"),
        ("sample_prepare", "wie_jit_sample_prepare"),
        ("word_range", "wie_jit_word_range"),
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
unsafe extern "C" fn wie_jit_load(access: u32, address: u32, width: u32, out: u32) -> u32 {
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
unsafe extern "C" fn wie_jit_store(access: u32, address: u32, width: u32, value: u32) -> u32 {
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
unsafe extern "C" fn wie_jit_sample_prepare(access: u32, pc: u32, cpsr: u32, r7: u32) {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    context.access.sample_prepare(pc, cpsr, r7, unsafe { (*context.frame).entry_pc });
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_jit_word_range(access: u32, address: u32, words: u32) -> u64 {
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
