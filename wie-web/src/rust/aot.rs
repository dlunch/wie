use alloc::{boxed::Box, collections::BTreeSet, format, rc::Rc, vec::Vec};
use core::cell::RefCell;

use futures::channel::oneshot;
use hashbrown::{HashMap, hash_map::Entry};
use js_sys::{Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use wie_arm_aot::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess, PreparationFuture, RunFrame,
    cache,
    manifest::{decode_manifest_region, encode_manifest_region, validate_manifest_region},
};

use wie_core_arm_wasm::{Compiler, WasmArtifact};
use wie_util::{Result as WieResult, WieError};

#[wasm_bindgen(inline_js = r#"
export { compileArm, yieldToMainThread, loadArmCache } from "@ts/arm-compiler.ts";
export function executeRegion(region, frame, context, slot) {
    const exit = region(frame, context, slot);
    // Reject non-numbers before the Wasm import can coerce them to valid exits.
    return typeof exit === "number" ? exit : NaN;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = compileArm)]
    fn compile_arm(
        bytes: &Uint8Array,
        record: &JsValue,
        key: &JsValue,
        cached_digest: &JsValue,
        imports: &Object,
        frame: u32,
        regions: u32,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(catch, js_name = loadArmCache)]
    fn load_arm_cache(input: &Uint8Array) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_name = yieldToMainThread)]
    fn yield_to_main_thread() -> Promise;

    #[wasm_bindgen(catch, js_name = executeRegion)]
    fn execute_region(region: &Function, frame: u32, context: u32, slot: u32) -> Result<f64, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

#[derive(Default)]
pub struct WasmExecutor {
    modules: Rc<RefCell<HashMap<u32, (Function, usize)>>>,
    next_module: u32,
}

// The browser host is single-agent: JS handles and synchronous borrowed execution
// contexts never leave its main thread.
unsafe impl Send for WasmExecutor {}

impl CompiledExecutor for WasmExecutor {
    fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
        let weak = Rc::downgrade(&self.modules);
        let module = self.next_module;
        self.next_module += 1;
        let (sender, receiver) = oneshot::channel();
        spawn_local(async move {
            let result = prepare_module(request, module).await;
            let Some(state) = weak.upgrade() else { return };
            let mut state = state.borrow_mut();
            let result = match result {
                Ok((artifact, dispatcher)) => {
                    if !artifact.regions.is_empty() {
                        state.insert(module, (dispatcher, artifact.regions.len()));
                    }
                    Ok(artifact)
                }
                Err(error) => Err(WieError::FatalError(format!("ARM AOT preparation: {error:?}"))),
            };
            if sender.send(result).is_err() {
                state.remove(&module);
            }
        });
        // Only the Send receiver crosses the host initialization future's await.
        Box::pin(async move { receiver.await.map_err(|_| WieError::FatalError("ARM AOT owner was dropped".into()))? })
    }

    fn release(&mut self, handle: CompiledHandle) {
        let mut modules = self.modules.borrow_mut();
        if let Entry::Occupied(mut entry) = modules.entry(handle.module) {
            entry.get_mut().1 -= 1;
            if entry.get().1 == 0 {
                entry.remove();
            }
        }
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> WieResult<CompiledExit> {
        let state = self.modules.borrow();
        let (function, _) = state
            .get(&handle.module)
            .ok_or_else(|| WieError::FatalError("compiled dispatcher is unavailable".into()))?;
        let mut context = ExecutionContext {
            access,
            frame,
            module: handle.module,
        };
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
                3.0 => Ok(CompiledExit::End),
                4.0 => Ok(CompiledExit::InterpretOne),
                6.0 => Ok(CompiledExit::GuestFault),
                _ => Err(WieError::FatalError(format!("compiled ABI returned invalid exit: {value:?}"))),
            },
            Err(error) => Err(WieError::FatalError(format!("generated code trapped: {error:?}"))),
        }
    }
}

async fn prepare_module(request: CompileRequest, module: u32) -> Result<(CompiledArtifact, Function), JsValue> {
    let mut group_started = now();
    // Persist only the initial image; runtime replacements belong to this execution.
    let lookup = if module == 0 {
        let input = Uint8Array::from(cache::key_input(&request).as_slice());
        JsFuture::from(load_arm_cache(&input)?).await?
    } else {
        Object::new().into()
    };
    let key = Reflect::get(&lookup, &"key".into())?;
    let cache_key: Option<[u8; 32]> = if key.is_undefined() {
        None
    } else {
        Some(key.clone().dyn_into::<Uint8Array>()?.to_vec().try_into().unwrap())
    };
    let candidate = Reflect::get(&lookup, &"artifact".into())?;
    if let Some(cache_key) = cache_key
        && !candidate.is_undefined()
    {
        let restored = async {
            let record = candidate.dyn_into::<Uint8Array>()?.to_vec();
            let (code, mut manifest) = cache::decode(&record, &cache_key).ok_or_else(|| JsValue::from_str("invalid cached artifact"))?;
            let mut owned = BTreeSet::new();
            let mut regions = Vec::new();
            while !manifest.is_empty() {
                preparation_checkpoint(&mut group_started).await?;
                let region = decode_manifest_region(&mut manifest).ok_or_else(|| JsValue::from_str("invalid cached manifest"))?;
                validate_manifest_region(&region, &request.images, &mut owned).map_err(|error| JsValue::from_str(&error))?;
                regions.push(CompiledRegion {
                    handle: CompiledHandle {
                        module,
                        slot: regions.len() as u32,
                    },
                    manifest: region,
                });
            }
            let digest = Reflect::get(&lookup, &"digest".into())?;
            let dispatcher = prepare_dispatcher(code, Some(&record), &key, &digest, regions.len()).await?;
            Ok::<_, JsValue>((CompiledArtifact { regions }, dispatcher))
        }
        .await;
        match restored {
            Ok(result) => return Ok(result),
            Err(error) => {
                tracing::warn!(?error, "ARM AOT cached artifact rejected; rebuilding");
            }
        }
    }

    let mut compiler = Compiler::new(request);
    loop {
        preparation_checkpoint(&mut group_started).await?;
        if compiler.step().map_err(|error| JsValue::from_str(&error))? {
            break;
        }
    }
    let WasmArtifact { bytes, manifest } = compiler.finish();
    let mut serialized = Vec::new();
    let mut regions = Vec::with_capacity(manifest.len());
    for (slot, manifest) in manifest.into_iter().enumerate() {
        preparation_checkpoint(&mut group_started).await?;
        encode_manifest_region(&manifest, &mut serialized);
        regions.push(CompiledRegion {
            manifest,
            handle: CompiledHandle { module, slot: slot as u32 },
        });
    }
    let record = cache_key.and_then(|key| cache::encode(&key, &bytes, &serialized));
    let dispatcher = prepare_dispatcher(&bytes, record.as_deref(), &key, &JsValue::UNDEFINED, regions.len()).await?;
    Ok((CompiledArtifact { regions }, dispatcher))
}

async fn preparation_checkpoint(group_started: &mut f64) -> Result<(), JsValue> {
    if now() - *group_started >= 4.0 {
        JsFuture::from(yield_to_main_thread()).await?;
        *group_started = now();
    }
    Ok(())
}

async fn prepare_dispatcher(
    bytes: &[u8],
    record: Option<&[u8]>,
    key: &JsValue,
    cached_digest: &JsValue,
    region_count: usize,
) -> Result<Function, JsValue> {
    let mut warmup = Box::new(RunFrame {
        cpsr: 0x1f,
        end: 0x1000,
        ..RunFrame::default()
    });
    warmup.regs[15] = 0x1000;
    let imports = execution_imports()?;
    // JS owns these bytes before an await can grow Wasm memory.
    let bytes = Uint8Array::from(bytes);
    let record = record.map(|record| JsValue::from(Uint8Array::from(record))).unwrap_or(JsValue::UNDEFINED);
    let promise = compile_arm(
        &bytes,
        &record,
        key,
        cached_digest,
        &imports,
        &mut *warmup as *mut RunFrame as u32,
        region_count as u32,
    )?;
    let result = JsFuture::from(promise).await;
    result?.dyn_into::<Function>()
}

fn execution_imports() -> Result<Object, JsValue> {
    let exports = wasm_bindgen::exports();
    let wie = Object::new();
    Reflect::set(&wie, &"memory".into(), &wasm_bindgen::memory())?;
    for (import, export) in [
        ("pages", "wie_aot_pages"),
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
    module: u32,
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
    assert!(core::mem::size_of::<wie_arm_aot::MemoryPage>() == 4);
    assert!(core::mem::offset_of!(wie_arm_aot::MemoryPage, bytes) == 0);
};

#[unsafe(no_mangle)]
unsafe extern "C" fn wie_aot_resolve(access: u32, pc: u32, cpsr: u32) -> u32 {
    let context = unsafe { &*(access as *const ExecutionContext<'_>) };
    // Other modules resume through the engine's entry map, never this module's local table.
    context
        .access
        .resolve(pc, cpsr)
        .filter(|handle| handle.module == context.module)
        .map_or(u32::MAX, |handle| handle.slot)
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
