use alloc::{boxed::Box, collections::BTreeSet, format, rc::Rc, string::String, vec, vec::Vec};
use core::cell::RefCell;

use futures::channel::oneshot;
use js_sys::{Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use wie_arm_jit_types::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess, PreparationFuture, RegionKey,
    RunFrame,
};

use crate::{AOT_CACHE_VERSION, Compiler, WasmArtifact, bind_manifest_source, decode_manifest_region, encode_manifest_region};

#[wasm_bindgen(inline_js = r#"
export { compileArm, compilerTask, loadArmCache } from "@ts/arm-compiler.ts";
export function executeRegion(region, frame, context, slot) {
    const exit = region(frame, context, slot);
    // Reject non-numbers before the Wasm import can coerce them to valid exits.
    return typeof exit === "number" ? exit : NaN;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = compileArm)]
    fn compile_arm(
        artifact: &Object,
        key: &JsValue,
        cached_digest: &JsValue,
        imports: &Object,
        frame: u32,
        regions: u32,
        deadline: f64,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_name = loadArmCache)]
    fn load_arm_cache(input: &Uint8Array, version: u32, deadline: f64) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_name = compilerTask)]
    fn compiler_task() -> Promise;

    #[wasm_bindgen(catch, js_name = executeRegion)]
    fn execute_region(region: &Function, frame: u32, context: u32, slot: u32) -> Result<f64, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    pub fn now() -> f64;
}

#[derive(Default)]
struct State {
    active: Vec<bool>,
    dispatcher: Option<Function>,
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
        let weak = Rc::downgrade(&self.state);
        let (sender, receiver) = oneshot::channel();
        spawn_local(async move {
            let result = prepare_module(request, deadline_ms).await;
            let Some(state) = weak.upgrade() else { return };
            let mut state = state.borrow_mut();
            let result = match result {
                Ok((artifact, dispatcher)) if now() < deadline_ms => {
                    state.active = vec![true; artifact.regions.len()];
                    state.dispatcher = Some(dispatcher);
                    Ok(artifact)
                }
                Ok(_) => Err(String::from("ARM AOT preparation timed out")),
                Err(error) => Err(format!("ARM AOT preparation: {error:?}")),
            };
            drop(state);
            let _ = sender.send(result);
        });
        // Only the Send receiver crosses the host initialization future's await.
        Box::pin(async move { receiver.await.map_err(|_| String::from("ARM AOT owner was dropped"))? })
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String> {
        let state = self.state.borrow();
        let function = state
            .dispatcher
            .as_ref()
            .filter(|_| state.active.get(handle.slot as usize) == Some(&true))
            .ok_or_else(|| String::from("compiled handle is retired or dispatcher is unavailable"))?;
        let mut context = ExecutionContext { access, frame };
        let result = execute_region(
            function,
            context.frame as u32,
            &mut context as *mut ExecutionContext<'_> as u32,
            handle.slot,
        );
        drop(state);
        match result {
            Ok(value) => match value {
                0.0 => Ok(CompiledExit::Dispatch),
                1.0 => Ok(CompiledExit::Sample),
                3.0 => Ok(CompiledExit::End),
                4.0 => Ok(CompiledExit::InterpretOne),
                6.0 => Ok(CompiledExit::GuestFault),
                _ => Err(format!("compiled ABI returned invalid exit: {value:?}")),
            },
            Err(error) => Err(format!("generated code trapped: {error:?}")),
        }
    }

    fn retire(&mut self, handles: &[CompiledHandle]) {
        let mut state = self.state.borrow_mut();
        for handle in handles {
            if let Some(active) = state.active.get_mut(handle.slot as usize) {
                *active = false;
            }
        }
    }
}

async fn prepare_module(request: CompileRequest, deadline: f64) -> Result<(CompiledArtifact, Function), JsValue> {
    let started = now();
    let memory_before = core::arch::wasm32::memory_size::<0>() * 65536;
    let mut cache = "miss";
    let mut input_ms = 0.0;
    let mut lookup_ms = 0.0;
    let mut compiler_ms = 0.0;
    let mut manifest_ms = 0.0;
    let mut output_setup_ms = 0.0;
    let mut encoded_size = 0;
    let mut region_count = 0;
    let result = async {
        let mut group_started = started;
        preparation_checkpoint(&mut group_started, deadline).await?;
        let mut input = Vec::new();
        input.extend_from_slice(&(request.images.len() as u32).to_le_bytes());
        for image in request.images.iter() {
            preparation_checkpoint(&mut group_started, deadline).await?;
            input.extend_from_slice(&image.address.to_le_bytes());
            input.extend_from_slice(&(image.bytes.len() as u32).to_le_bytes());
            input.extend_from_slice(&image.bytes);
        }
        let input_copy = Uint8Array::from(input.as_slice());
        drop(input);
        input_ms = now() - started;
        preparation_checkpoint(&mut group_started, deadline).await?;
        let lookup_started = now();
        let lookup = JsFuture::from(load_arm_cache(&input_copy, AOT_CACHE_VERSION, deadline)?).await?;
        drop(input_copy);
        lookup_ms = now() - lookup_started;
        preparation_checkpoint(&mut group_started, deadline).await?;
        let key = Reflect::get(&lookup, &"key".into())?;
        if key.is_undefined() {
            cache = "unavailable";
        }
        let candidate = match Reflect::get(&lookup, &"artifact".into())? {
            candidate if candidate.is_undefined() => None,
            candidate => Some(candidate.dyn_into::<Object>()?),
        };
        drop(lookup);
        if let Some(artifact) = candidate {
            let restored = async {
                let manifest_started = now();
                let restored = async {
                    let bytes = Reflect::get(&artifact, &"manifest".into())?.dyn_into::<Uint8Array>()?.to_vec();
                    let mut manifest = bytes.as_slice();
                    let mut owned = BTreeSet::new();
                    let mut regions = Vec::new();
                    while !manifest.is_empty() {
                        preparation_checkpoint(&mut group_started, deadline).await?;
                        let mut region = decode_manifest_region(&mut manifest).ok_or_else(|| JsValue::from_str("invalid cached manifest"))?;
                        bind_manifest_source(&mut region, &request.images).map_err(|error| JsValue::from_str(&error))?;
                        for &pc in &region.instruction_pcs {
                            if !owned.insert(RegionKey { pc, ..region.entry }) {
                                return Err(JsValue::from_str("duplicate cached instruction ownership"));
                            }
                        }
                        regions.push(CompiledRegion {
                            handle: CompiledHandle { slot: regions.len() as u32 },
                            manifest: region,
                        });
                    }
                    Ok(regions)
                }
                .await;
                manifest_ms += now() - manifest_started;
                preparation_checkpoint(&mut group_started, deadline).await?;
                let regions = restored?;
                encoded_size = Reflect::get(&artifact, &"bytes".into())?.dyn_into::<Uint8Array>()?.length() as usize;
                let digest = Reflect::get(&artifact, &"digest".into())?;
                region_count = regions.len();
                cache = "persistent-hit";
                let dispatcher = prepare_dispatcher(&artifact, &key, &digest, region_count, deadline, &mut output_setup_ms).await?;
                Ok::<_, JsValue>((CompiledArtifact { regions, encoded_size }, dispatcher))
            }
            .await;
            preparation_checkpoint(&mut group_started, deadline).await?;
            match restored {
                Ok(result) => return Ok(result),
                Err(error) => {
                    cache = "corrupt";
                    tracing::warn!(?error, "ARM AOT cached artifact rejected; rebuilding");
                }
            }
        }

        let compiler_started = now();
        let mut compiler = Compiler::new(request);
        let compiled = async {
            loop {
                preparation_checkpoint(&mut group_started, deadline).await?;
                if compiler.step().map_err(|error| JsValue::from_str(&error))? {
                    break;
                }
            }
            Ok::<_, JsValue>(compiler.finish())
        }
        .await;
        compiler_ms = now() - compiler_started;
        let WasmArtifact { bytes, manifest } = compiled?;
        encoded_size = bytes.len();
        let manifest_started = now();
        let mut serialized = Vec::new();
        let mut regions = Vec::with_capacity(manifest.len());
        for (slot, manifest) in manifest.into_iter().enumerate() {
            preparation_checkpoint(&mut group_started, deadline).await?;
            encode_manifest_region(&manifest, &mut serialized);
            regions.push(CompiledRegion {
                manifest,
                handle: CompiledHandle { slot: slot as u32 },
            });
        }
        manifest_ms += now() - manifest_started;
        preparation_checkpoint(&mut group_started, deadline).await?;
        let output_started = now();
        let artifact = Object::new();
        // Both arrays own their bytes before Rust allocations are freed or an await can grow memory.
        let bytes_copy = Uint8Array::from(bytes.as_slice());
        drop(bytes);
        Reflect::set(&artifact, &"bytes".into(), &bytes_copy)?;
        let manifest_copy = Uint8Array::from(serialized.as_slice());
        drop(serialized);
        Reflect::set(&artifact, &"manifest".into(), &manifest_copy)?;
        output_setup_ms += now() - output_started;
        region_count = regions.len();
        preparation_checkpoint(&mut group_started, deadline).await?;
        let dispatcher = prepare_dispatcher(&artifact, &key, &JsValue::UNDEFINED, region_count, deadline, &mut output_setup_ms).await?;
        preparation_checkpoint(&mut group_started, deadline).await?;
        Ok((CompiledArtifact { regions, encoded_size }, dispatcher))
    }
    .await;
    let elapsed_ms = now() - started;
    let memory_retained = core::arch::wasm32::memory_size::<0>() * 65536;
    tracing::info!(
        cache,
        input_ms,
        lookup_ms,
        compiler_ms,
        manifest_ms,
        output_setup_ms,
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

async fn preparation_checkpoint(group_started: &mut f64, deadline: f64) -> Result<(), JsValue> {
    let current = now();
    if current >= deadline {
        return Err(JsValue::from_str("ARM AOT preparation timed out"));
    }
    if current - *group_started >= 4.0 {
        JsFuture::from(compiler_task()).await?;
        *group_started = now();
        if *group_started >= deadline {
            return Err(JsValue::from_str("ARM AOT preparation timed out"));
        }
    }
    Ok(())
}

async fn prepare_dispatcher(
    artifact: &Object,
    key: &JsValue,
    cached_digest: &JsValue,
    region_count: usize,
    deadline: f64,
    output_setup_ms: &mut f64,
) -> Result<Function, JsValue> {
    let output_started = now();
    let mut warmup = Box::new(RunFrame {
        cpsr: 0x1f,
        end: 0x1000,
        sample_remaining: 1,
        ..RunFrame::default()
    });
    warmup.regs[15] = 0x1000;
    let imports = execution_imports()?;
    *output_setup_ms += now() - output_started;
    let promise = compile_arm(
        artifact,
        key,
        cached_digest,
        &imports,
        &mut *warmup as *mut RunFrame as u32,
        region_count as u32,
        deadline,
    )?;
    let result = JsFuture::from(promise).await;
    // TS checks job settlement after every await, so timed-out continuations cannot reuse this frame.
    drop(warmup);
    result?.dyn_into::<Function>()
}

fn execution_imports() -> Result<Object, JsValue> {
    let exports = wasm_bindgen::exports();
    let wie = Object::new();
    Reflect::set(&wie, &"memory".into(), &wasm_bindgen::memory())?;
    for (import, export) in [
        ("pages", "wie_aot_pages"),
        ("sample_prepare", "wie_aot_sample_prepare"),
        ("word_range", "wie_aot_word_range"),
        ("resolve", "wie_aot_resolve"),
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
unsafe extern "C" fn wie_aot_pages(access: u32) -> u32 {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    context.access.pages().as_mut_ptr() as u32
}

#[cfg(target_arch = "wasm32")]
const _: () = {
    assert!(core::mem::size_of::<wie_arm_jit_types::MemoryPage>() == 16);
    assert!(core::mem::offset_of!(wie_arm_jit_types::MemoryPage, bytes) == 0);
};

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_sample_prepare(access: u32, pc: u32, r7: u32) {
    let context = unsafe { &mut *(access as *mut ExecutionContext<'_>) };
    context.access.sample_prepare(pc, r7);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_resolve(access: u32, pc: u32, cpsr: u32) -> u32 {
    let context = unsafe { &*(access as *const ExecutionContext<'_>) };
    context.access.resolve(pc, cpsr).map_or(u32::MAX, |handle| handle.slot)
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
