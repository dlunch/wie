#![cfg(all(not(target_os = "ios"), any(target_arch = "x86_64", target_arch = "aarch64")))]

use std::{
    collections::BTreeMap,
    future::poll_fn,
    mem::transmute,
    ops::Range,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};

use cranelift_codegen::ir::{AbiParam, UserFuncName, types};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use spin::Mutex;

use wie_arm_jit_types::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess, ManifestRegion, MemoryPage,
    PreparationFuture, RunFrame, ir::RegionIr,
};
use wie_util::{Result, WieError};

mod codegen;

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

struct NativeRegion {
    _module: ModuleOwner,
    entry: RegionFn,
}

#[derive(Default)]
struct Modules {
    next_id: u32,
    regions: BTreeMap<u32, NativeRegion>,
}

pub struct NativeExecutor {
    modules: Arc<Mutex<Modules>>,
    origin: Instant,
}

impl NativeExecutor {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(Mutex::new(Modules::default())),
            origin: Instant::now(),
        }
    }
}

impl Default for NativeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CompiledExecutor for NativeExecutor {
    fn now(&self) -> f64 {
        self.origin.elapsed().as_secs_f64() * 1000.0
    }

    fn prepare(&mut self, request: CompileRequest, deadline_ms: f64) -> PreparationFuture {
        let modules = Arc::downgrade(&self.modules);
        let origin = self.origin;
        let mut regions = request.regions;
        Box::pin(async move {
            let mut compiled = Vec::new();
            poll_fn(|context| {
                if modules.upgrade().is_none() {
                    return Poll::Ready(Err(WieError::FatalError("Native ARM AOT owner was dropped".into())));
                }
                let started = Instant::now();
                loop {
                    if origin.elapsed().as_secs_f64() * 1000.0 >= deadline_ms {
                        return Poll::Ready(Err(WieError::FatalError("Native ARM AOT preparation timed out".into())));
                    }
                    let done = match regions.next() {
                        Some(Some(region)) => {
                            compiled.push(compile_region(region.ir)?);
                            false
                        }
                        Some(None) => false,
                        None => true,
                    };
                    if origin.elapsed().as_secs_f64() * 1000.0 >= deadline_ms {
                        return Poll::Ready(Err(WieError::FatalError("Native ARM AOT preparation timed out".into())));
                    }
                    if done {
                        return Poll::Ready(Ok(()));
                    }
                    if started.elapsed() >= Duration::from_millis(4) {
                        context.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            })
            .await?;
            let modules = modules
                .upgrade()
                .ok_or_else(|| WieError::FatalError("Native ARM AOT owner was dropped".into()))?;
            let mut modules = modules.lock();
            let mut artifact = CompiledArtifact {
                regions: Vec::with_capacity(compiled.len()),
            };
            for (region, manifest) in compiled {
                let module = modules.next_id;
                modules.next_id += 1;
                modules.regions.insert(module, region);
                artifact.regions.push(CompiledRegion {
                    manifest,
                    handle: CompiledHandle { module, slot: 0 },
                });
            }
            Ok(artifact)
        })
    }

    fn release(&mut self, handle: CompiledHandle) {
        self.modules.lock().regions.remove(&handle.module);
    }

    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
        let modules = self.modules.lock();
        let region = modules
            .regions
            .get(&handle.module)
            .ok_or_else(|| WieError::FatalError("Native ARM AOT region is unavailable".into()))?;
        // Finalized code has this exact ABI; both borrows and the executable owner
        // remain live throughout the synchronous call, without host callbacks.
        let exit = unsafe { (region.entry)(frame, access.pages().as_mut_ptr()) };
        match exit {
            0 => Ok(CompiledExit::Dispatch),
            3 => Ok(CompiledExit::End),
            4 => Ok(CompiledExit::InterpretOne),
            6 => Ok(CompiledExit::GuestFault),
            _ => Err(WieError::FatalError(format!("Native ARM AOT returned invalid exit {exit}"))),
        }
    }
}

fn compile_region(ir: RegionIr) -> Result<(NativeRegion, ManifestRegion)> {
    let builder =
        JITBuilder::new(default_libcall_names()).map_err(|error| WieError::FatalError(format!("Creating native ARM AOT compiler: {error}")))?;
    let mut owner = ModuleOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().unwrap();
    let mut context = module.make_context();
    let pointer = module.isa().pointer_type();
    context.func.signature.params.extend([AbiParam::new(pointer), AbiParam::new(pointer)]);
    context.func.signature.returns.push(AbiParam::new(types::I32));
    let function = module
        .declare_function("region", Linkage::Local, &context.func.signature)
        .map_err(|error| WieError::FatalError(format!("Declaring native ARM AOT region: {error}")))?;
    context.func.name = UserFuncName::user(0, function.as_u32());
    codegen::emit_region(&ir, &mut context, module.isa())?;
    module
        .define_function(function, &mut context)
        .map_err(|error| WieError::FatalError(format!("Compiling native ARM AOT region: {error}")))?;
    module
        .finalize_definitions()
        .map_err(|error| WieError::FatalError(format!("Finalizing native ARM AOT region: {error}")))?;
    // The module uses the host C calling convention and the RegionFn signature.
    let entry = unsafe { transmute::<*const u8, RegionFn>(module.get_finalized_function(function)) };

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
        NativeRegion { _module: owner, entry },
        ManifestRegion {
            entry: ir.entry,
            instruction_pcs,
            code_ranges,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Context, Poll, Waker},
    };

    use cranelift_codegen::ir::{AbiParam, InstBuilder, types};
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_jit::{BranchProtection, JITBuilder, JITMemoryKind, JITMemoryProvider, JITModule, SystemMemoryProvider};
    use cranelift_module::{Linkage, Module, ModuleResult, default_libcall_names};
    use futures::executor::block_on;
    use wie_arm_jit_types::{
        CompileRegion, CompileRequest, CompiledExecutor, CompiledExit, CompiledHandle, ExecutionAccess, MemoryPage, RegionKey, RunFrame,
        ir::{AluOp, BasicBlock, Condition, Instruction, MemoryAddress, MemoryOperand, Operand, Operation, Reg, RegionIr, Shift, ShiftAmount, Value},
    };

    use super::{ModuleOwner, NativeExecutor};

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

    #[test]
    fn preparation_preserves_exact_coverage_and_region_lifetimes() {
        let mut executor = NativeExecutor::new();
        let artifact = block_on(
            executor.prepare(
                request(
                    [
                        Some(region(0x1004, &[&[(0x1004, 4)], &[(0x1000, 4)], &[(0x1010, 4)]])),
                        Some(region(0x2000, &[&[(0x2000, 4)]])),
                    ]
                    .into_iter(),
                ),
                executor.now() + 10_000.0,
            ),
        )
        .unwrap();
        assert_eq!(artifact.regions.len(), 2);
        assert_eq!(artifact.regions[0].manifest.instruction_pcs, [0x1000, 0x1004, 0x1010]);
        assert_eq!(artifact.regions[0].manifest.code_ranges, [0x1000..0x1008, 0x1010..0x1014]);
        assert_eq!(artifact.regions[0].manifest.entry.pc, 0x1004);
        let first = artifact.regions[0].handle;
        let second = artifact.regions[1].handle;
        assert_ne!(first.module, second.module);
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
            budget: 2,
            ..RunFrame::default()
        };
        frame.regs[15] = 0x1000;
        assert!(executor.execute(first, &mut frame, &mut access).unwrap() == CompiledExit::End);
        assert_eq!(frame.executed, 2);
        assert_eq!(frame.regs[15], 0x1008);
        executor.release(first);
        assert!(executor.execute(first, &mut frame, &mut access).is_err());
        assert_eq!(frame.executed, 2);
        assert_eq!(frame.regs[15], 0x1008);
        frame.regs[15] = 0x2000;
        frame.end = 0x2004;
        frame.budget = 1;
        frame.executed = 0;
        assert!(executor.execute(second, &mut frame, &mut access).unwrap() == CompiledExit::End);
        assert_eq!(frame.executed, 1);
        executor.release(second);
        assert!(executor.modules.lock().regions.is_empty());
    }

    #[test]
    fn unfinished_preparation_yields_and_cleans_up_on_cancellation_or_owner_loss() {
        for drop_owner in [false, true] {
            let mut executor = NativeExecutor::new();
            let owner = Arc::downgrade(&executor.modules);
            let steps = Arc::new(AtomicUsize::new(0));
            let observed = steps.clone();
            let iterator_lifetime = Arc::new(());
            let iterator_owner = Arc::downgrade(&iterator_lifetime);
            let regions = std::iter::once(Some(region(0x1000, &[&[(0x1000, 4)]])))
                .chain(std::iter::repeat_with(|| None))
                .inspect(move |_| {
                    let _ = &iterator_lifetime;
                    observed.fetch_add(1, Ordering::Relaxed);
                });
            let mut future = executor.prepare(request(regions), executor.now() + 10_000.0);
            let mut context = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut context).is_pending());
            assert!(steps.load(Ordering::Relaxed) >= 1);
            assert!(executor.modules.lock().regions.is_empty());
            if drop_owner {
                drop(executor);
                assert!(owner.upgrade().is_none());
                assert!(matches!(future.as_mut().poll(&mut context), Poll::Ready(Err(_))));
                assert!(iterator_owner.upgrade().is_none());
            } else {
                drop(future);
                assert!(iterator_owner.upgrade().is_none());
                assert!(executor.modules.lock().regions.is_empty());
            }
        }
    }

    #[test]
    fn expired_preparation_does_not_decode_or_publish() {
        let mut executor = NativeExecutor::new();
        let input = request(std::iter::from_fn(|| panic!("expired preparation must not decode")));
        assert!(block_on(executor.prepare(input, 0.0)).is_err());
        assert!(executor.modules.lock().regions.is_empty());
    }

    #[test]
    fn malformed_register_fails_preparation_without_publishing_a_prefix() {
        let mut executor = NativeExecutor::new();
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
            let mut invalid = region(0x2000, &[&[(0x2000, 4)]]);
            invalid.ir.blocks[0].instructions[0].operation = operation;
            let input = request([Some(region(0x1000, &[&[(0x1000, 4)]])), Some(invalid)].into_iter());
            assert!(block_on(executor.prepare(input, executor.now() + 10_000.0)).is_err());
            assert!(executor.modules.lock().regions.is_empty());
        }
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
                    let run = unsafe { std::mem::transmute::<*const u8, unsafe extern "C" fn() -> u32>(module.get_finalized_function(function)) };
                    assert_eq!(unsafe { run() }, 7);
                }
                assert_eq!(frees.load(Ordering::Relaxed), 0);
            }
            assert_eq!(frees.load(Ordering::Relaxed), 1);
        }
    }
}
