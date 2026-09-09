use alloc::{
    collections::{BTreeMap, VecDeque},
    format,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::cell::RefCell;

use js_sys::{Function, Object, Reflect, WebAssembly};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{ErrorEvent, MessageEvent, Worker};
use wie_arm_jit::{
    AccessResult, Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion,
    ExecutionAccess, ManifestRegion, RunFrame,
};

const RESERVATION: usize = 512 * 1024;
const LIVE_BYTES: usize = 2 * 1024 * 1024;
const PEAK_BYTES: usize = 4 * 1024 * 1024;
const PENDING_IR_BYTES: usize = 1024 * 1024;

#[wasm_bindgen(module = "/src/bootstrap.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = createCompilerWorker)]
    fn create_compiler_worker() -> Result<Worker, JsValue>;
}

struct Pending {
    request: CompileRequest,
    payload: String,
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
            if id.as_deref() != Some(active.pending.request.request.to_string().as_str()) {
                return;
            }
            let decoded = decode_response(&data, &active.pending.request);
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
            let request = active.pending.request.request;
            let count = active.pending.request.regions.len();
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
                if active.pending.request.request != request || !matches!(active.stage, Stage::Installing) {
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
        let Some(pending) = state.queue.pop_front() else { return };
        let stage = if let Some(error) = &state.worker_failure {
            Stage::Ready(Err(error.clone()))
        } else {
            let message = Object::new();
            let sent = (|| {
                Reflect::set(&message, &"request".into(), &pending.request.request.to_string().into())?;
                Reflect::set(&message, &"payload".into(), &JsValue::from_str(&pending.payload))?;
                worker.post_message(&message)
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
    fn submit(&mut self, request: CompileRequest) -> Admission {
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
        let payload = match serde_json::to_string(&request) {
            Ok(payload) => payload,
            Err(error) => return Admission::Failed(error.to_string()),
        };
        // Include core/runtime/worker IR and UTF-8/UTF-16 transport copies.
        let request_cost = 4 * request.ir_size() + 5 * payload.len();
        if request_cost > PENDING_IR_BYTES {
            return Admission::Failed(String::from("compile request exceeds the IR budget"));
        }
        let pending_cost: usize = state
            .queue
            .iter()
            .chain(state.active.as_ref().map(|active| &active.pending))
            .map(|pending| 4 * pending.request.ir_size() + 5 * pending.payload.len())
            .sum();
        if pending_cost + request_cost > PENDING_IR_BYTES {
            return Admission::Busy;
        }
        let pending = state.queue.len() + usize::from(state.active.is_some());
        let bytes: usize = state.modules.values().map(|module| module.encoded_size).sum();
        if state.queue.len() >= 4 || state.modules.len() + pending + 1 > 12 || bytes + (pending + 1) * RESERVATION > PEAK_BYTES {
            return Admission::Busy;
        }
        if !fits_live(&state, &request, RESERVATION) {
            return Admission::Busy;
        }
        state.queue.push_back(Pending { request, payload });
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
        let request = active.pending.request;
        let session = request.session;
        let request_id = request.request;
        let result = result.and_then(|installed| {
            if !fits_live(&state, &request, installed.encoded_size) {
                return Err(String::from("compiled installation exceeds the live module budget"));
            }
            let encoded_size = installed.encoded_size;
            let regions = request
                .regions
                .into_iter()
                .enumerate()
                .map(|(index, region)| CompiledRegion {
                    manifest: ManifestRegion {
                        entry: region.ir.entry,
                        source: region.source,
                        export: format!("region_{index}"),
                        expected_old: region.expected_old,
                    },
                    handle: CompiledHandle {
                        slot: index as u32,
                        generation: request.request,
                    },
                })
                .collect();
            state.modules.insert(request.request, installed);
            Ok(CompiledArtifact { regions, encoded_size })
        });
        let completion = CompileCompletion {
            session,
            request: request_id,
            result,
        };
        drop(state);
        self.start_next();
        Some(completion)
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String> {
        let function = self
            .state
            .borrow()
            .modules
            .get(&handle.generation)
            .and_then(|module| module.functions.get(handle.slot as usize))
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| String::from("compiled handle is retired or executor is closed"))?;
        let mut context = ExecutionContext { access, frame };
        let result = function.call2(
            &JsValue::UNDEFINED,
            &JsValue::from(context.frame as u32),
            &JsValue::from(&mut context as *mut ExecutionContext<'_> as u32),
        );
        let exit = match result {
            Ok(value) => match value.as_f64() {
                Some(0.0) => Ok(CompiledExit::Dispatch),
                Some(1.0) => Ok(CompiledExit::Sample),
                Some(2.0) => Ok(CompiledExit::Budget),
                Some(3.0) => Ok(CompiledExit::End),
                Some(4.0) => Ok(CompiledExit::InterpretOne),
                Some(5.0) => Ok(CompiledExit::Invalidated),
                Some(6.0) => Ok(CompiledExit::GuestFault),
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

fn fits_live(state: &State, request: &CompileRequest, output_size: usize) -> bool {
    let mut count = 1;
    let mut bytes = output_size;
    for (&generation, module) in &state.modules {
        let replaced = module.functions.iter().enumerate().all(|(slot, function)| {
            function.is_none()
                || request.regions.iter().any(|region| {
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
    count <= 8 && bytes <= LIVE_BYTES
}

fn decode_response(data: &JsValue, request: &CompileRequest) -> Result<(WebAssembly::Module, usize), String> {
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
    if manifest.len() != request.regions.len()
        || manifest.iter().zip(&request.regions).enumerate().any(|(index, (actual, expected))| {
            actual.entry != expected.ir.entry
                || actual.source != expected.source
                || actual.expected_old != expected.expected_old
                || actual.export != format!("region_{index}")
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
        AccessResult::Invalidated => 3,
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
        AccessResult::Invalidated => 3,
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_jit_sample_prepare(access: u32, pc: u32, cpsr: u32, r7: u32) {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    context.access.sample_prepare(pc, cpsr, r7);
}
