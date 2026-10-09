#![no_std]
extern crate alloc;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, btree_map::Entry},
    format,
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{mem::transmute, ops::Range};

use cranelift_codegen::{
    Context,
    ir::{AbiParam, UserFuncName, types},
};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Module, default_libcall_names};
use futures::channel::oneshot;
use rayon::iter::{ParallelBridge, ParallelIterator};
use spin::Mutex;

use wie_arm_jit_types::{
    CodeImage, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess, ManifestRegion,
    MemoryPage, PreparationFuture, RunFrame, ir::RegionIr,
};
use wie_util::{Result, WieError};

mod cache;
mod codegen;

pub use cache::NativeCache;

type RegionFn = unsafe extern "C" fn(*mut RunFrame, *mut MemoryPage) -> u32;

struct ModuleOwner(Option<JITModule>);

impl Drop for ModuleOwner {
    fn drop(&mut self) {
        if let Some(module) = self.0.take() {
            // Functions are borrowed only while their registry entry is locked.
            unsafe { module.free_memory() };
        }
    }
}

struct NativeModule {
    _module: ModuleOwner,
    entries: BTreeMap<u32, RegionFn>,
}

struct Compiler {
    module: ModuleOwner,
    context: Context,
    builder_context: FunctionBuilderContext,
    regions: Vec<PendingRegion>,
}

struct PendingRegion {
    index: usize,
    function: FuncId,
    manifest: ManifestRegion,
    code: Option<(u32, Vec<u8>)>,
}

#[derive(Default)]
struct Modules {
    next_id: u32,
    compiled: BTreeMap<u32, NativeModule>,
}

pub struct NativeExecutor {
    modules: Arc<Mutex<Modules>>,
    cache: Option<Box<dyn NativeCache>>,
}

impl NativeExecutor {
    pub fn new(cache: Option<Box<dyn NativeCache>>) -> Self {
        Self {
            modules: Arc::new(Mutex::new(Modules::default())),
            cache,
        }
    }
}

impl CompiledExecutor for NativeExecutor {
    fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
        let modules = Arc::downgrade(&self.modules);
        let cache = self.cache.take();
        let (sender, receiver) = oneshot::channel();
        rayon::spawn(move || {
            let prepared = (|| -> Result<_> {
                let cache = if let Some(mut cache) = cache {
                    let module = create_module()?;
                    let key = cache::key(module.0.as_ref().unwrap().isa(), &request);
                    match cache.load(&key) {
                        Ok(Some(bytes)) => match restore(module, &bytes, &key, &request.images) {
                            Ok(compiled) => {
                                tracing::info!(bytes = bytes.len(), "Native ARM AOT persistent cache hit");
                                return Ok((compiled, None));
                            }
                            Err(error) => tracing::warn!("Rejecting native ARM AOT cache: {error}"),
                        },
                        Ok(None) => tracing::debug!("Native ARM AOT cache miss"),
                        Err(error) => tracing::warn!("Reading native ARM AOT cache: {error}"),
                    }
                    Some((cache, key))
                } else {
                    None
                };
                let compiled = compile(request, cache.is_some())?;
                let pending = cache
                    .and_then(|(cache, key)| cache::encode(&key, compiled.iter().flat_map(|(_, regions)| regions)).map(|bytes| (cache, key, bytes)));
                Ok((compiled, pending))
            })();
            let _ = sender.send(prepared);
        });
        Box::pin(async move {
            let (compiled, pending) = receiver
                .await
                .map_err(|error| WieError::FatalError(format!("Receiving native ARM AOT compilation: {error}")))??;
            let modules = modules
                .upgrade()
                .ok_or_else(|| WieError::FatalError("Native ARM AOT owner was dropped".into()))?;
            let mut regions = Vec::with_capacity(compiled.iter().map(|(_, regions)| regions.len()).sum());
            {
                let mut modules = modules.lock();
                for (owner, compiled) in compiled {
                    let module = owner.0.as_ref().unwrap();
                    let id = modules.next_id;
                    modules.next_id += 1;
                    let mut entries = BTreeMap::new();
                    for (slot, region) in compiled.into_iter().enumerate() {
                        // Each entry has the host C RegionFn ABI and stays owned until the last region is released.
                        let entry = unsafe { transmute::<*const u8, RegionFn>(module.get_finalized_function(region.function)) };
                        entries.insert(slot as u32, entry);
                        regions.push((
                            region.index,
                            CompiledRegion {
                                manifest: region.manifest,
                                handle: CompiledHandle {
                                    module: id,
                                    slot: slot as u32,
                                },
                            },
                        ));
                    }
                    modules.compiled.insert(id, NativeModule { _module: owner, entries });
                }
            }
            regions.sort_unstable_by_key(|(index, _)| *index);
            if let Some((mut cache, key, bytes)) = pending {
                rayon::spawn(move || {
                    // The complete record comes only from successfully finalized compiler output.
                    match unsafe { cache.store(&key, &bytes) } {
                        Ok(()) => tracing::info!(bytes = bytes.len(), "Stored native ARM AOT cache"),
                        Err(error) => tracing::warn!("Writing native ARM AOT cache: {error}"),
                    }
                });
            }
            Ok(CompiledArtifact {
                regions: regions.into_iter().map(|(_, region)| region).collect(),
            })
        })
    }

    fn release(&mut self, handle: CompiledHandle) {
        let mut modules = self.modules.lock();
        if let Entry::Occupied(mut entry) = modules.compiled.entry(handle.module) {
            entry.get_mut().entries.remove(&handle.slot);
            if entry.get().entries.is_empty() {
                entry.remove();
            }
        }
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
        let modules = self.modules.lock();
        let entry = modules
            .compiled
            .get(&handle.module)
            .and_then(|module| module.entries.get(&handle.slot))
            .ok_or_else(|| WieError::FatalError("Native ARM AOT region is unavailable".into()))?;
        // Finalized code has this exact ABI; both borrows and the executable owner
        // remain live throughout the synchronous call; generated code cannot reenter execution or remap pages.
        let exit = unsafe { entry(frame, access.pages().as_mut_ptr()) };
        match exit {
            0 => Ok(CompiledExit::Dispatch),
            6 => Ok(CompiledExit::GuestFault),
            _ => Err(WieError::FatalError(format!("Native ARM AOT returned invalid exit {exit}"))),
        }
    }
}

fn create_module() -> Result<ModuleOwner> {
    let builder = JITBuilder::with_flags(&[("enable_verifier", "false")], default_libcall_names())
        .map_err(|error| WieError::FatalError(format!("Creating native ARM AOT compiler: {error}")))?;
    Ok(ModuleOwner(Some(JITModule::new(builder))))
}

fn compile(request: CompileRequest, cache: bool) -> Result<Vec<(ModuleOwner, Vec<PendingRegion>)>> {
    request
        .coalesced_regions()
        .flatten()
        .enumerate()
        .par_bridge()
        .try_fold(
            || None,
            |compiler: Option<Compiler>, (index, region)| -> Result<_> {
                let mut compiler = match compiler {
                    Some(compiler) => compiler,
                    None => {
                        let module = create_module()?;
                        let context = module.0.as_ref().unwrap().make_context();
                        Compiler {
                            module,
                            context,
                            builder_context: FunctionBuilderContext::new(),
                            regions: Vec::new(),
                        }
                    }
                };
                let (function, manifest) = compile_region(
                    compiler.module.0.as_mut().unwrap(),
                    &mut compiler.context,
                    &mut compiler.builder_context,
                    region.ir,
                )?;
                let code = if cache {
                    let code = compiler.context.compiled_code().unwrap();
                    if !code.buffer.relocs().is_empty() {
                        return Err(WieError::FatalError("Native ARM AOT cache requires relocation-free code".into()));
                    }
                    Some((code.buffer.alignment, code.code_buffer().to_vec()))
                } else {
                    None
                };
                compiler.regions.push(PendingRegion {
                    index,
                    function,
                    manifest,
                    code,
                });
                Ok(Some(compiler))
            },
        )
        .filter_map(|compiler| compiler.transpose())
        .map(|compiler| -> Result<_> {
            let mut compiler = compiler?;
            compiler
                .module
                .0
                .as_mut()
                .unwrap()
                .finalize_definitions()
                .map_err(|error| WieError::FatalError(format!("Finalizing native ARM AOT module: {error}")))?;
            Ok((compiler.module, compiler.regions))
        })
        .collect()
}

fn restore(mut owner: ModuleOwner, bytes: &[u8], key: &[u8; 32], images: &[CodeImage]) -> Result<Vec<(ModuleOwner, Vec<PendingRegion>)>> {
    let cached = cache::decode(bytes, key, images).ok_or_else(|| WieError::FatalError("Invalid native ARM AOT cache record".into()))?;
    if cached.is_empty() {
        return Ok(Vec::new());
    }
    let module = owner.0.as_mut().unwrap();
    let mut signature = module.make_signature();
    signature.params.extend([AbiParam::new(module.isa().pointer_type()); 2]);
    signature.returns.push(AbiParam::new(types::I32));
    let mut regions = Vec::with_capacity(cached.len());
    for (index, region) in cached.into_iter().enumerate() {
        let function = module
            .declare_anonymous_function(&signature)
            .map_err(|error| WieError::FatalError(format!("Declaring cached native ARM AOT region: {error}")))?;
        module
            .define_function_bytes(function, u64::from(region.alignment), region.code, &[])
            .map_err(|error| WieError::FatalError(format!("Loading native ARM AOT region: {error}")))?;
        regions.push(PendingRegion {
            index,
            function,
            manifest: region.manifest,
            code: None,
        });
    }
    module
        .finalize_definitions()
        .map_err(|error| WieError::FatalError(format!("Finalizing cached native ARM AOT module: {error}")))?;
    Ok(vec![(owner, regions)])
}

fn compile_region(
    module: &mut JITModule,
    context: &mut Context,
    builder_context: &mut FunctionBuilderContext,
    ir: RegionIr,
) -> Result<(FuncId, ManifestRegion)> {
    module.clear_context(context);
    let pointer = module.isa().pointer_type();
    context.func.signature.params.extend([AbiParam::new(pointer), AbiParam::new(pointer)]);
    context.func.signature.returns.push(AbiParam::new(types::I32));
    let function = module
        .declare_anonymous_function(&context.func.signature)
        .map_err(|error| WieError::FatalError(format!("Declaring native ARM AOT region: {error}")))?;
    context.func.name = UserFuncName::user(0, function.as_u32());
    codegen::emit_region(&ir, context, builder_context, module.isa())?;
    module
        .define_function(function, context)
        .map_err(|error| WieError::FatalError(format!("Compiling native ARM AOT region: {error}")))?;

    let mut instructions: Vec<_> = ir.blocks.iter().flat_map(|block| &block.instructions).collect();
    instructions.sort_unstable_by_key(|instruction| instruction.pc.get());
    let instruction_pcs = instructions.iter().map(|instruction| instruction.pc.get()).collect();
    let mut code_ranges: Vec<Range<u64>> = Vec::new();
    for instruction in instructions {
        let start = u64::from(instruction.pc.get());
        let end = start + u64::from(instruction.size);
        if let Some(previous) = code_ranges.last_mut()
            && start <= previous.end
        {
            previous.end = previous.end.max(end);
        } else {
            code_ranges.push(start..end);
        }
    }
    Ok((
        function,
        ManifestRegion {
            entry: ir.entry,
            instruction_pcs,
            code_ranges,
        },
    ))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{boxed::Box, collections::BTreeSet, sync::Arc, vec, vec::Vec};
    use core::{
        mem::transmute,
        sync::atomic::{AtomicUsize, Ordering},
        task::{Context, Waker},
    };
    use std::{io, sync::mpsc, time::Duration};

    use cranelift_codegen::ir::{AbiParam, InstBuilder, types};
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_jit::{BranchProtection, JITBuilder, JITMemoryKind, JITMemoryProvider, JITModule, SystemMemoryProvider};
    use cranelift_module::{Linkage, Module, ModuleResult, default_libcall_names};
    use futures::executor::block_on;
    use rayon::ThreadPoolBuilder;
    use spin::Mutex;
    use wie_arm_jit_types::{
        CodeImage, CompileRegion, CompileRequest, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, MemoryPage, RegionKey, RunFrame,
        ir::{AluOp, BasicBlock, Condition, Instruction, MemoryAddress, MemoryOperand, Operand, Operation, Reg, RegionIr, Shift, ShiftAmount, Value},
    };
    use wie_util::{Result, WieError};

    use super::{ModuleOwner, NativeCache, NativeExecutor};

    fn region(entry: u32, blocks: &[&[(u32, u8)]]) -> CompileRegion {
        CompileRegion {
            ir: RegionIr {
                entry: RegionKey {
                    pc: entry,
                    thumb: false,
                    cpu_mode: 0x1f,
                },
                blocks: blocks
                    .iter()
                    .map(|instructions| BasicBlock {
                        instructions: instructions
                            .iter()
                            .map(|&(pc, size)| Instruction {
                                pc: MemoryAddress::new(pc),
                                size,
                                condition: Condition::Always,
                                operation: Operation::Nop,
                            })
                            .collect(),
                    })
                    .collect(),
            },
        }
    }

    fn request(regions: impl Iterator<Item = Option<CompileRegion>> + Send + 'static) -> CompileRequest {
        CompileRequest {
            images: Arc::from([]),
            max_region_instructions: 4096,
            max_region_blocks: 512,
            regions: Box::new(regions),
        }
    }

    struct Access(Box<[MemoryPage; 0x10000]>);

    impl ExecutionAccess for Access {
        fn resolve(&self, _: u32, _: u32) -> Option<CompiledHandle> {
            panic!("native execution must return to the engine for dispatch")
        }

        fn pages(&mut self) -> &mut [MemoryPage; 0x10000] {
            &mut self.0
        }

        fn word_range(&mut self, _: u32, _: u32) -> Option<(&mut [u8], &mut [u8])> {
            panic!("native memory operations must not call host helpers")
        }
    }

    struct TestCache {
        bytes: Arc<Mutex<Option<Vec<u8>>>>,
        calls: Arc<[AtomicUsize; 2]>,
        stored: mpsc::Sender<()>,
        fail: bool,
    }

    // Only this test's compiler output is loaded; corruption cases fail validation before execution.
    unsafe impl NativeCache for TestCache {
        fn load(&mut self, _: &[u8; 32]) -> Result<Option<Vec<u8>>> {
            assert!(rayon::current_thread_index().is_some());
            self.calls[0].fetch_add(1, Ordering::Relaxed);
            if self.fail {
                return Err(WieError::FatalError("unavailable cache".into()));
            }
            Ok(self.bytes.lock().clone())
        }

        unsafe fn store(&mut self, _: &[u8; 32], bytes: &[u8]) -> Result<()> {
            assert!(rayon::current_thread_index().is_some());
            self.calls[1].fetch_add(1, Ordering::Relaxed);
            if !self.fail {
                *self.bytes.lock() = Some(bytes.to_vec());
            }
            self.stored.send(()).unwrap();
            if self.fail {
                return Err(WieError::FatalError("unavailable cache".into()));
            }
            Ok(())
        }
    }

    #[test]
    fn persistent_preparation_restores_without_decoding_and_rebuilds_rejected_entries() {
        let bytes: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
        let calls = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
        let images: Arc<[_]> = [CodeImage {
            address: 0x1000,
            bytes: vec![0; 12],
        }]
        .into();
        for attempt in 0..5 {
            let hit = attempt == 1;
            if attempt == 2 {
                bytes.lock().as_mut().unwrap()[0] ^= 1;
            }
            let (stored, completion) = mpsc::channel();
            let cache = TestCache {
                bytes: bytes.clone(),
                calls: calls.clone(),
                stored,
                fail: attempt == 4,
            };
            let mut executor = NativeExecutor::new(Some(Box::new(cache)));
            let mut instructions = region(0x1000, &[&[(0x1000, 4), (0x1004, 4), (0x1008, 4)]]);
            for (instruction, operation) in instructions.ir.blocks[0].instructions.iter_mut().zip([
                Operation::Alu {
                    op: AluOp::Add,
                    destination: Some(Reg::new(0)),
                    left: Value::Register(Reg::new(0)),
                    right: Operand {
                        value: Value::Immediate(1),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    set_flags: true,
                },
                Operation::Alu {
                    op: AluOp::Move,
                    destination: Some(Reg::new(1)),
                    left: Value::Immediate(0),
                    right: Operand {
                        value: Value::Register(Reg::new(0)),
                        shift: Shift::Rrx,
                        amount: ShiftAmount::Immediate(0),
                    },
                    set_flags: true,
                },
                Operation::Alu {
                    op: AluOp::Move,
                    destination: Some(Reg::new(2)),
                    left: Value::Immediate(0),
                    right: Operand {
                        value: Value::Register(Reg::new(1)),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Register(Reg::new(3)),
                    },
                    set_flags: true,
                },
            ]) {
                instruction.operation = operation;
            }
            let mut input = request(
                [Some(instructions)]
                    .into_iter()
                    .inspect(move |_| assert!(!hit, "persistent hit must not advance the decoder")),
            );
            input.images = images.clone();
            if attempt == 3 {
                input.max_region_blocks += 1;
            }
            let artifact = block_on(executor.prepare(input)).unwrap();
            assert_eq!(artifact.regions.len(), 1);
            assert_eq!(artifact.regions[0].manifest.instruction_pcs, [0x1000, 0x1004, 0x1008]);
            assert_eq!(artifact.regions[0].manifest.code_ranges.len(), 1);
            assert_eq!(artifact.regions[0].manifest.code_ranges[0], 0x1000..0x100c);
            let mut access = Access(
                (0..0x10000)
                    .map(|_| MemoryPage::default())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
                    .try_into()
                    .ok()
                    .unwrap(),
            );
            let mut frame = RunFrame {
                cpsr: 0x1f,
                ..RunFrame::default()
            };
            frame.regs[15] = 0x1000;
            frame.regs[0] = u32::MAX;
            let handle = artifact.regions[0].handle;
            assert!(executor.execute(handle, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
            assert_eq!(frame.regs[15], 0x100c);
            assert_eq!(&frame.regs[..4], [0, 0x8000_0000, 0x8000_0000, 0]);
            assert_eq!(frame.cpsr, 0x8000_001f);
            executor.release(handle);
            assert!(executor.modules.lock().compiled.is_empty());
            if !hit {
                completion.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            let reads = calls[0].load(Ordering::Relaxed);
            let writes = calls[1].load(Ordering::Relaxed);
            block_on(executor.prepare(request([].into_iter()))).unwrap();
            assert_eq!(calls[0].load(Ordering::Relaxed), reads);
            assert_eq!(calls[1].load(Ordering::Relaxed), writes);
        }
        assert_eq!(calls[0].load(Ordering::Relaxed), 5);
        assert_eq!(calls[1].load(Ordering::Relaxed), 4);
    }

    #[test]
    fn preparation_preserves_exact_coverage_and_region_lifetimes() {
        for workers in [1, 2] {
            let pool = ThreadPoolBuilder::new().num_threads(workers).build().unwrap();
            let calls = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
            let (stored, completion) = mpsc::channel();
            let mut executor = NativeExecutor::new(Some(Box::new(TestCache {
                bytes: Arc::new(Mutex::new(None)),
                calls: calls.clone(),
                stored,
                fail: false,
            })));
            for input in [request([].into_iter()), request([None, None].into_iter())] {
                let future = pool.install(|| executor.prepare(input));
                assert!(block_on(future).unwrap().regions.is_empty());
                assert!(executor.modules.lock().compiled.is_empty());
            }
            completion.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(calls[0].load(Ordering::Relaxed), 1);
            assert_eq!(calls[1].load(Ordering::Relaxed), 1);
            let mut mode_change = region(0x4000, &[&[(0x4000, 4), (0x4004, 4)]]);
            mode_change.ir.blocks[0].instructions[0].operation = Operation::WriteCpsr {
                value: Value::Register(Reg::new(0)),
                mask: 0x0100_003f,
            };
            let input = request(
                [
                    Some(region(0x1004, &[&[(0x1004, 4)], &[(0x1000, 4)], &[(0x1010, 4)]])),
                    None,
                    Some(region(0x1014, &[&[(0x1014, 4)]])),
                    Some(mode_change),
                ]
                .into_iter()
                .chain((2..8).map(|index| {
                    let pc = index * 0x4000;
                    Some(region(pc, &[&[(pc, 4)]]))
                })),
            );
            let future = pool.install(|| executor.prepare(input));
            let artifact = block_on(future).unwrap();
            assert_eq!(artifact.regions.len(), 8);
            assert_eq!(
                artifact.regions.iter().map(|region| region.manifest.entry.pc).collect::<Vec<_>>(),
                [0x1004, 0x4000, 0x8000, 0xc000, 0x10000, 0x14000, 0x18000, 0x1c000]
            );
            assert_eq!(artifact.regions[0].manifest.instruction_pcs, [0x1000, 0x1004, 0x1010, 0x1014]);
            assert_eq!(artifact.regions[0].manifest.code_ranges, [0x1000..0x1008, 0x1010..0x1018]);
            assert_eq!(artifact.regions[1].manifest.instruction_pcs, [0x4000, 0x4004]);
            assert_eq!(artifact.regions[1].manifest.code_ranges.len(), 1);
            assert_eq!(artifact.regions[1].manifest.code_ranges[0], 0x4000..0x4008);
            assert_eq!(
                artifact
                    .regions
                    .iter()
                    .map(|region| (region.handle.module, region.handle.slot))
                    .collect::<BTreeSet<_>>()
                    .len(),
                artifact.regions.len()
            );
            let first = artifact.regions[0].handle;
            let second = artifact.regions[1].handle;
            let mut access = Access(
                (0..0x10000)
                    .map(|_| MemoryPage::default())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
                    .try_into()
                    .ok()
                    .unwrap(),
            );
            let mut frame = RunFrame {
                cpsr: 0x1f,
                end: 0x1008,
                ..RunFrame::default()
            };
            for pc in [0x1001, 0x1002, 0x1003] {
                frame.regs[15] = pc;
                assert!(executor.execute(first, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
                assert_eq!(frame.regs[15], pc);
            }
            frame.regs[15] = 0x1000;
            assert!(executor.execute(first, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
            assert_eq!(frame.regs[15], 0x1008);
            frame.regs[15] = 0x1014;
            frame.end = 0x1018;
            assert!(executor.execute(first, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
            assert_eq!(frame.regs[15], 0x1018);
            executor.release(first);
            assert!(executor.modules.lock().compiled[&second.module].entries.contains_key(&second.slot));
            assert!(executor.execute(first, &mut frame, &mut access).is_err());
            assert_eq!(frame.regs[15], 0x1018);
            for cpsr in [0x13, 0x3f, 0x0100_001f] {
                frame.cpsr = 0x1f;
                frame.regs[0] = cpsr;
                frame.regs[15] = 0x4000;
                frame.end = 0x4008;
                assert!(executor.execute(second, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
                assert_eq!(frame.cpsr, cpsr);
                assert_eq!(frame.regs[15], 0x4004);
            }
            frame.cpsr = 0x1f;
            assert!(executor.execute(second, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
            assert_eq!(frame.regs[15], 0x4008);
            executor.release(second);
            for region in &artifact.regions[2..] {
                let pc = region.manifest.entry.pc;
                assert_eq!(region.manifest.instruction_pcs, [pc]);
                assert_eq!(region.manifest.code_ranges.len(), 1);
                assert_eq!(region.manifest.code_ranges[0], u64::from(pc)..u64::from(pc + 4));
                frame.regs[15] = pc;
                frame.end = pc + 4;
                assert!(executor.execute(region.handle, &mut frame, &mut access).unwrap() == CompiledExit::Dispatch);
                assert_eq!(frame.regs[15], pc + 4);
                executor.release(region.handle);
            }
            assert!(executor.modules.lock().compiled.is_empty());
            assert_eq!(calls[0].load(Ordering::Relaxed), 1);
            assert_eq!(calls[1].load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn preparation_runs_on_rayon_and_waits_for_completion_before_publication() {
        for workers in [1, 2] {
            let pool = ThreadPoolBuilder::new().num_threads(workers).build().unwrap();
            for drop_owner in [false, true] {
                let calls = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
                let (stored, completion) = mpsc::channel();
                let mut executor = NativeExecutor::new(Some(Box::new(TestCache {
                    bytes: Arc::new(Mutex::new(None)),
                    calls: calls.clone(),
                    stored,
                    fail: false,
                })));
                let owner = Arc::downgrade(&executor.modules);
                let steps = Arc::new(AtomicUsize::new(0));
                let observed = steps.clone();
                let iterator_lifetime = Arc::new(());
                let iterator_owner = Arc::downgrade(&iterator_lifetime);
                let (entered, entry) = mpsc::channel();
                let (release, released) = mpsc::channel();
                let regions = [Some(region(0x1000, &[&[(0x1000, 4)]])), Some(region(0x4000, &[&[(0x4000, 4)]]))]
                    .into_iter()
                    .inspect(move |_| {
                        let _ = &iterator_lifetime;
                        if observed.fetch_add(1, Ordering::Relaxed) == 0 {
                            let _ = entered.send((rayon::current_thread_index(), rayon::current_num_threads()));
                            let _ = released.recv_timeout(Duration::from_secs(10));
                        }
                    });
                let mut future = pool.install(|| executor.prepare(request(regions)));
                let (worker, pool_size) = entry.recv_timeout(Duration::from_secs(5)).expect("preparation must start before polling");
                assert!(worker.is_some());
                assert_eq!(pool_size, workers);
                let mut context = Context::from_waker(Waker::noop());
                assert!(future.as_mut().poll(&mut context).is_pending());
                assert!(executor.modules.lock().compiled.is_empty());
                if drop_owner {
                    drop(executor);
                    assert!(owner.upgrade().is_none());
                    assert!(future.as_mut().poll(&mut context).is_pending());
                    release.send(()).unwrap();
                    assert!(block_on(future).is_err());
                    assert!(completion.recv_timeout(Duration::from_secs(5)).is_err());
                    assert_eq!(calls[1].load(Ordering::Relaxed), 0);
                } else {
                    release.send(()).unwrap();
                    let artifact = block_on(future).unwrap();
                    assert_eq!(artifact.regions.len(), 2);
                    for region in artifact.regions {
                        executor.release(region.handle);
                    }
                    assert!(executor.modules.lock().compiled.is_empty());
                    completion.recv_timeout(Duration::from_secs(5)).unwrap();
                    assert_eq!(calls[1].load(Ordering::Relaxed), 1);
                }
                assert_eq!(calls[0].load(Ordering::Relaxed), 1);
                assert_eq!(steps.load(Ordering::Relaxed), 2);
                assert!(iterator_owner.upgrade().is_none());
            }
        }
    }

    #[test]
    fn malformed_register_fails_preparation_without_publishing_a_prefix() {
        let pool = ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        let calls = Arc::new([AtomicUsize::new(0), AtomicUsize::new(0)]);
        let (stored, completion) = mpsc::channel();
        let mut executor = NativeExecutor::new(Some(Box::new(TestCache {
            bytes: Arc::new(Mutex::new(None)),
            calls: calls.clone(),
            stored,
            fail: false,
        })));
        for operation in [
            Operation::ReadCpsr { destination: Reg::new(255) },
            Operation::Alu {
                op: AluOp::Move,
                destination: Some(Reg::new(0)),
                left: Value::Immediate(0),
                right: Operand {
                    value: Value::Immediate(1),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Register(Reg::new(255)),
                },
                set_flags: false,
            },
            Operation::DoubleTransfer {
                register: Reg::new(15),
                address: MemoryOperand {
                    base: Value::Immediate(0x10000),
                    offset: Operand {
                        value: Value::Immediate(0),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    subtract: false,
                    pre_index: true,
                    write_back: None,
                },
                load: true,
            },
        ] {
            let mut invalid = region(0x4000, &[&[(0x4000, 4)]]);
            invalid.ir.blocks[0].instructions[0].operation = operation;
            let input = request([Some(region(0x1000, &[&[(0x1000, 4)]])), Some(invalid)].into_iter());
            let future = pool.install(|| executor.prepare(input));
            assert!(block_on(future).is_err());
            assert!(executor.modules.lock().compiled.is_empty());
            let future = pool.install(|| executor.prepare(request([Some(region(0x1000, &[&[(0x1000, 4)]]))].into_iter())));
            let artifact = block_on(future).unwrap();
            assert_eq!(artifact.regions.len(), 1);
            assert_eq!(artifact.regions[0].manifest.instruction_pcs, [0x1000]);
            executor.release(artifact.regions[0].handle);
            assert!(executor.modules.lock().compiled.is_empty());
        }
        assert!(completion.recv_timeout(Duration::from_secs(5)).is_err());
        assert_eq!(calls[0].load(Ordering::Relaxed), 1);
        assert_eq!(calls[1].load(Ordering::Relaxed), 0);
    }

    #[test]
    fn module_owner_frees_unfinished_and_finalized_allocations() {
        struct ObservedMemory {
            inner: SystemMemoryProvider,
            frees: Arc<AtomicUsize>,
        }

        impl JITMemoryProvider for ObservedMemory {
            fn allocate(&mut self, size: usize, align: u64, kind: JITMemoryKind) -> io::Result<*mut u8> {
                self.inner.allocate(size, align, kind)
            }

            unsafe fn free_memory(&mut self) {
                unsafe { self.inner.free_memory() };
                self.frees.fetch_add(1, Ordering::Relaxed);
            }

            fn finalize(&mut self, protection: BranchProtection) -> ModuleResult<()> {
                self.inner.finalize(protection)
            }
        }

        for finalize in [false, true] {
            let frees = Arc::new(AtomicUsize::new(0));
            {
                let mut builder = JITBuilder::new(default_libcall_names()).unwrap();
                builder.memory_provider(Box::new(ObservedMemory {
                    inner: SystemMemoryProvider::new(),
                    frees: frees.clone(),
                }));
                let mut owner = ModuleOwner(Some(JITModule::new(builder)));
                let module = owner.0.as_mut().unwrap();
                let mut context = module.make_context();
                context.func.signature.returns.push(AbiParam::new(types::I32));
                let function = module.declare_function("owned", Linkage::Local, &context.func.signature).unwrap();
                let mut builder_context = FunctionBuilderContext::new();
                let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
                let entry = builder.create_block();
                builder.switch_to_block(entry);
                let value = builder.ins().iconst(types::I32, 7);
                builder.ins().return_(&[value]);
                builder.seal_all_blocks();
                builder.finalize(module.isa().frontend_config());
                module.define_function(function, &mut context).unwrap();
                if finalize {
                    module.finalize_definitions().unwrap();
                    let run = unsafe { transmute::<*const u8, unsafe extern "C" fn() -> u32>(module.get_finalized_function(function)) };
                    assert_eq!(unsafe { run() }, 7);
                }
                assert_eq!(frees.load(Ordering::Relaxed), 0);
            }
            assert_eq!(frees.load(Ordering::Relaxed), 1);
        }
    }
}
