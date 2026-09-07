use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, sync::Arc, vec, vec::Vec};
use core::mem::size_of;

use spin::Mutex;

use wie_backend::{ProfileCallback, ProfileSample, YieldFuture};
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use crate::{
    EmulatedFunction, ResultWriter, ThreadId,
    context::ArmCoreContext,
    engine::{Arm32CpuEngine, ArmEngine, ArmRegister, EngineStopReason, MemoryPermission},
    function::{RegisteredFunction, RegisteredFunctionHolder},
    thread::ThreadState,
    thread_wrapper::ArmCoreThreadWrapper,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    engine::{DebugInner, DebuggedArm32CpuEngine},
    gdb::GdbTarget,
};

const GLOBAL_DATA_BASE: u32 = 0x7fff0000;
const FUNCTIONS_BASE: u32 = 0x71000000;
const FUNCTIONS_SIZE: usize = 0x10000;
const SVC_STUB_SIZE: u32 = 16;
const INSTRUCTIONS_PER_YIELD: u32 = 10_000;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const HEAP_SIZE: u32 = 0x10000000;

/// Limit on stack frames recorded per sample. Bounds memory and protects
/// against runaway loops if the R7 chain forms a cycle.
const PROFILE_MAX_STACK: usize = 32;
/// Flush the per-stack counter map every this many samples taken.
const PROFILE_FLUSH_INTERVAL: u32 = 1000;

struct ProfileState {
    samples: BTreeMap<Vec<u32>, u64>,
    counter: u32,
    callback: ProfileCallback,
}

pub(crate) struct ArmCoreInner {
    pub(crate) engine: Box<dyn ArmEngine>,
    instructions_remaining: u32,
    last_thread_id: ThreadId,
    svc_handlers: BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>,
    next_stub_address: u32,
    profile: Option<ProfileState>,
}

impl Drop for ArmCoreInner {
    fn drop(&mut self) {
        if let Some(mut profile) = self.profile.take() {
            let batch = drain_samples(&mut profile.samples);
            if !batch.is_empty() {
                (profile.callback)(batch);
            }
        }
    }
}

fn drain_samples(samples: &mut BTreeMap<Vec<u32>, u64>) -> Vec<ProfileSample> {
    core::mem::take(samples)
        .into_iter()
        .map(|(stack, count)| ProfileSample { stack, count })
        .collect()
}

#[derive(Clone)]
pub struct ArmCore {
    pub(crate) inner: Arc<Mutex<ArmCoreInner>>, // TODO can we change it to another lock like async-lock?
    threads: Arc<Mutex<BTreeMap<ThreadId, ThreadState>>>,
}

impl ArmCore {
    pub fn new(enable_gdbserver: bool, profile: Option<ProfileCallback>) -> Result<Self> {
        let mut engine = if enable_gdbserver {
            #[cfg(not(target_arch = "wasm32"))]
            let engine = Box::new(DebuggedArm32CpuEngine::new()) as Box<dyn ArmEngine>;
            #[cfg(target_arch = "wasm32")]
            let engine = Box::new(Arm32CpuEngine::new());

            engine
        } else {
            Box::new(Arm32CpuEngine::new())
        };

        engine.mem_map(FUNCTIONS_BASE, FUNCTIONS_SIZE, MemoryPermission::ReadExecute);
        engine.mem_map(GLOBAL_DATA_BASE, 0x4000, MemoryPermission::ReadWriteExecute);

        let profile = profile.map(|callback| ProfileState {
            samples: BTreeMap::new(),
            counter: 0,
            callback,
        });

        let inner = ArmCoreInner {
            engine,
            instructions_remaining: INSTRUCTIONS_PER_YIELD,
            last_thread_id: 0,
            svc_handlers: BTreeMap::new(),
            next_stub_address: FUNCTIONS_BASE,
            profile,
        };

        let result = Self {
            inner: Arc::new(Mutex::new(inner)),
            threads: Arc::new(Mutex::new(BTreeMap::new())),
        };

        if enable_gdbserver {
            #[cfg(not(target_arch = "wasm32"))]
            GdbTarget::start(result.clone())?;
            #[cfg(target_arch = "wasm32")]
            panic!("GDB server is not supported on wasm32");
        }

        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn debug_inner(&self) -> Option<Arc<DebugInner>> {
        let inner = self.inner.lock();

        inner
            .engine
            .as_any()
            .downcast_ref::<DebuggedArm32CpuEngine>()
            .map(|engine| engine.debug_inner())
    }

    pub fn load(&mut self, data: &[u8], address: u32, map_size: usize) -> Result<()> {
        let mut inner = self.inner.lock();

        inner
            .engine
            .mem_map(address, map_size.next_multiple_of(0x1000), MemoryPermission::ReadWriteExecute);
        inner.engine.mem_write(address, data)?;

        Ok(())
    }

    pub fn run_in_thread<F, Fut>(&self, entry: F) -> Result<ArmCoreThreadWrapper>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let state = ThreadState::new(self.clone())?;

        let thread_id = {
            let mut inner = self.inner.lock();

            let thread_id = inner.last_thread_id + 1;
            inner.last_thread_id += 1;

            thread_id
        };
        self.threads.lock().insert(thread_id, state);

        tracing::info!("Create thread: {thread_id}");

        ArmCoreThreadWrapper::new(self.clone(), thread_id, entry)
    }

    pub fn delete_thread_context(&self, thread_id: ThreadId) {
        tracing::info!("Terminate thread: {thread_id}");

        // Release the thread lock before freeing its stack.
        let _thread_state = self.threads.lock().remove(&thread_id);
    }

    pub fn enter_thread_context(&self, thread_id: ThreadId) -> ThreadContextGuard {
        ThreadContextGuard::new(self.clone(), thread_id)
    }

    pub fn read_thread_context(&self, thread_id: ThreadId) -> Result<ArmCoreContext> {
        let context = self.threads.lock().get(&thread_id).unwrap().context.clone();

        Ok(context)
    }

    pub fn write_thread_context(&mut self, thread_id: ThreadId, context: &ArmCoreContext) {
        self.threads.lock().get_mut(&thread_id).unwrap().context = context.clone();
    }

    pub fn get_thread_ids(&self) -> Vec<ThreadId> {
        self.threads.lock().keys().cloned().collect()
    }

    fn sample_profile(&self) {
        let mut inner = self.inner.lock();
        if inner.profile.is_none() {
            return;
        }
        let pc = inner.engine.reg_read(ArmRegister::PC);
        let mut stack = vec![pc];
        let mut r7 = inner.engine.reg_read(ArmRegister::R7);
        for _ in 0..PROFILE_MAX_STACK {
            // Thumb frame: [saved R7 | saved LR] at [r7].
            let mut buf = [0u8; 8];
            if inner.engine.mem_read(r7, 8, &mut buf).is_err() {
                break;
            }
            let prev_r7 = u32::from_le_bytes(buf[0..4].try_into().unwrap());
            let lr = u32::from_le_bytes(buf[4..8].try_into().unwrap());
            // Heuristic stop: zero/null frame or non-Thumb LR (we only ever
            // call into Thumb code from the guest).
            if prev_r7 == 0 || lr == 0 || lr & 1 == 0 {
                break;
            }
            stack.push(lr);
            if prev_r7 <= r7 {
                // R7 chain must walk upward; bail on any inversion to avoid loops.
                break;
            }
            r7 = prev_r7;
        }
        let profile = inner.profile.as_mut().unwrap();
        *profile.samples.entry(stack).or_insert(0) += 1;
        profile.counter = profile.counter.wrapping_add(1);
        if profile.counter >= PROFILE_FLUSH_INTERVAL {
            profile.counter = 0;
            let batch = drain_samples(&mut profile.samples);
            (profile.callback)(batch);
        }
    }

    pub async fn run_function<R>(&mut self, address: u32, params: &[u32]) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        // we don't need to save r0-r3, but to make it simple, we save all registers
        let previous_context = self.save_context();
        {
            let mut inner = self.inner.lock();

            if !params.is_empty() {
                inner.engine.reg_write(ArmRegister::R0, params[0]);
            }
            if params.len() > 1 {
                inner.engine.reg_write(ArmRegister::R1, params[1]);
            }
            if params.len() > 2 {
                inner.engine.reg_write(ArmRegister::R2, params[2]);
            }
            if params.len() > 3 {
                inner.engine.reg_write(ArmRegister::R3, params[3]);
            }
            if params.len() > 4 {
                for param in params[4..].iter().rev() {
                    let sp: u32 = inner.engine.reg_read(ArmRegister::SP) - 4;

                    inner.engine.mem_write(sp, &param.to_le_bytes())?;
                    inner.engine.reg_write(ArmRegister::SP, sp);
                }
            }

            inner.engine.reg_write(ArmRegister::PC, address);
            inner.engine.reg_write(ArmRegister::LR, RUN_FUNCTION_LR);

            let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
            let new_cpsr = (cpsr & !0x3f) | 0x1f | ((address & 1) << 5);
            inner.engine.reg_write(ArmRegister::Cpsr, new_cpsr);
        }

        loop {
            let (result, should_yield) = {
                let mut inner = self.inner.lock();
                let budget = inner.instructions_remaining;
                let result = inner.engine.run(RUN_FUNCTION_LR, budget)?;
                inner.instructions_remaining -= result.instructions_executed;
                let exhausted = inner.instructions_remaining == 0;
                if exhausted {
                    inner.instructions_remaining = INSTRUCTIONS_PER_YIELD;
                }
                let should_yield = exhausted || matches!(result.stop_reason, EngineStopReason::Yield);
                (result.stop_reason, should_yield)
            };

            self.sample_profile();

            if let EngineStopReason::Svc { lr, spsr, .. } = result {
                // Leave exception mode before yielding: thread contexts do not save banked SVC registers.
                let mut inner = self.inner.lock();
                inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                inner.engine.reg_write(ArmRegister::PC, lr);
            }

            if should_yield {
                YieldFuture::new().await;
            }

            match result {
                EngineStopReason::End => break,
                EngineStopReason::Yield => continue,
                EngineStopReason::Svc { category, .. } => {
                    let function = {
                        let inner = self.inner.lock();
                        inner
                            .svc_handlers
                            .get(&category)
                            .cloned()
                            .ok_or_else(|| WieError::FatalError(format!("Unknown SVC handler category: {category}")))?
                    };

                    let mut self1 = self.clone();
                    function.call(&mut self1).await?;
                }
            }
        }

        let result = R::get(self);
        self.restore_context(&previous_context);

        Ok(result)
    }

    pub fn register_svc_handler<F, C, R, P>(&mut self, category: u32, handler: F, context: &C) -> Result<()>
    where
        F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
        C: Clone + 'static + Sync + Send,
        R: ResultWriter<R> + Sync + Send + 'static,
        P: Sync + Send + 'static,
    {
        let mut inner = self.inner.lock();

        if inner.svc_handlers.contains_key(&category) {
            return Err(WieError::FatalError(format!("SVC handler already registered for {category}")));
        }

        inner
            .svc_handlers
            .insert(category, Arc::new(Box::new(RegisteredFunctionHolder::new(handler, context))));

        Ok(())
    }

    pub fn make_svc_stub(&mut self, category: u32, id: impl Into<u32>) -> Result<u32> {
        let mut inner = self.inner.lock();
        let id = id.into();

        if !inner.svc_handlers.contains_key(&category) {
            return Err(WieError::FatalError(format!("Unknown SVC handler category: {category}")));
        }

        let address = inner.next_stub_address;
        if address + SVC_STUB_SIZE > FUNCTIONS_BASE + FUNCTIONS_SIZE as u32 {
            return Err(WieError::FatalError("SVC stub space exhausted".into()));
        }
        inner.next_stub_address += SVC_STUB_SIZE;

        let stub = [
            0x10,
            0xb4, // push {r4}
            0x02,
            0x4c, // ldr r4, [pc, #8]
            0xa4,
            0x46, // mov r12, r4
            0x10,
            0xbc, // pop {r4}
            category as u8,
            0xdf, // svc #category
            0x70,
            0x47, // bx lr
        ]
        .into_iter()
        .chain(id.to_le_bytes())
        .collect::<Vec<_>>();
        inner.engine.mem_write(address, &stub)?;

        tracing::trace!("Register SVC stub at {address:#x}, category={category}, id={id}");

        Ok(address + 1)
    }

    pub fn map(&mut self, address: u32, size: u32) -> Result<()> {
        tracing::trace!("Map address: {address:#x}, size: {size:#x}");

        let mut inner = self.inner.lock();

        inner.engine.mem_map(address, size as usize, MemoryPermission::ReadWrite);

        Ok(())
    }

    pub fn dump_reg_stack(&self, image_base: u32) -> String {
        format!(
            "\n{}\nPossible call stack:\n{}\nStack:\n{}",
            self.dump_regs(),
            self.dump_call_stack(image_base).unwrap(),
            self.dump_stack().unwrap()
        )
    }

    pub fn restore_context(&mut self, context: &ArmCoreContext) {
        let mut inner = self.inner.lock();

        inner.engine.reg_write(ArmRegister::R0, context.r0);
        inner.engine.reg_write(ArmRegister::R1, context.r1);
        inner.engine.reg_write(ArmRegister::R2, context.r2);
        inner.engine.reg_write(ArmRegister::R3, context.r3);
        inner.engine.reg_write(ArmRegister::R4, context.r4);
        inner.engine.reg_write(ArmRegister::R5, context.r5);
        inner.engine.reg_write(ArmRegister::R6, context.r6);
        inner.engine.reg_write(ArmRegister::R7, context.r7);
        inner.engine.reg_write(ArmRegister::R8, context.r8);
        inner.engine.reg_write(ArmRegister::SB, context.sb);
        inner.engine.reg_write(ArmRegister::SL, context.sl);
        inner.engine.reg_write(ArmRegister::FP, context.fp);
        inner.engine.reg_write(ArmRegister::IP, context.ip);
        inner.engine.reg_write(ArmRegister::SP, context.sp);
        inner.engine.reg_write(ArmRegister::LR, context.lr);
        inner.engine.reg_write(ArmRegister::PC, context.pc);
        inner.engine.reg_write(ArmRegister::Cpsr, context.cpsr);
    }

    pub fn save_context(&self) -> ArmCoreContext {
        let inner = self.inner.lock();

        ArmCoreContext {
            r0: inner.engine.reg_read(ArmRegister::R0),
            r1: inner.engine.reg_read(ArmRegister::R1),
            r2: inner.engine.reg_read(ArmRegister::R2),
            r3: inner.engine.reg_read(ArmRegister::R3),
            r4: inner.engine.reg_read(ArmRegister::R4),
            r5: inner.engine.reg_read(ArmRegister::R5),
            r6: inner.engine.reg_read(ArmRegister::R6),
            r7: inner.engine.reg_read(ArmRegister::R7),
            r8: inner.engine.reg_read(ArmRegister::R8),
            sb: inner.engine.reg_read(ArmRegister::SB),
            sl: inner.engine.reg_read(ArmRegister::SL),
            fp: inner.engine.reg_read(ArmRegister::FP),
            ip: inner.engine.reg_read(ArmRegister::IP),
            sp: inner.engine.reg_read(ArmRegister::SP),
            lr: inner.engine.reg_read(ArmRegister::LR),
            pc: inner.engine.reg_read(ArmRegister::PC),
            cpsr: inner.engine.reg_read(ArmRegister::Cpsr),
        }
    }

    pub fn read_pc_lr(&self) -> Result<(u32, u32)> {
        let inner = self.inner.lock();

        let lr = inner.engine.reg_read(ArmRegister::LR);
        let pc = inner.engine.reg_read(ArmRegister::PC);

        Ok((pc, lr))
    }

    pub fn write_return_value(&mut self, result: &[u32]) -> Result<()> {
        let mut inner = self.inner.lock();

        if !result.is_empty() {
            inner.engine.reg_write(ArmRegister::R0, result[0]);
        }
        if result.len() > 1 {
            inner.engine.reg_write(ArmRegister::R1, result[1]);
        }
        if result.len() > 2 {
            todo!() // TODO
        }

        Ok(())
    }

    pub fn set_next_pc(&mut self, pc: u32) -> Result<()> {
        let mut inner = self.inner.lock();

        inner.engine.reg_write(ArmRegister::PC, pc);

        let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
        let new_cpsr = if pc & 1 == 1 { cpsr | 0x20 } else { cpsr & !0x20 };
        inner.engine.reg_write(ArmRegister::Cpsr, new_cpsr);

        Ok(())
    }

    pub fn read_param(&self, pos: usize) -> Result<u32> {
        let inner = self.inner.lock();

        let result = if pos == 0 {
            inner.engine.reg_read(ArmRegister::R0)
        } else if pos == 1 {
            inner.engine.reg_read(ArmRegister::R1)
        } else if pos == 2 {
            inner.engine.reg_read(ArmRegister::R2)
        } else if pos == 3 {
            inner.engine.reg_read(ArmRegister::R3)
        } else {
            let sp = inner.engine.reg_read(ArmRegister::SP);

            drop(inner);

            read_generic(self, sp + 4 * (pos as u32 - 4))?
        };

        Ok(result)
    }

    pub(crate) fn dump_regs_inner(engine: &dyn ArmEngine) -> String {
        [
            format!(
                "R0: {:#x} R1: {:#x} R2: {:#x} R3: {:#x} R4: {:#x} R5: {:#x} R6: {:#x} R7: {:#x} R8: {:#x}",
                engine.reg_read(ArmRegister::R0),
                engine.reg_read(ArmRegister::R1),
                engine.reg_read(ArmRegister::R2),
                engine.reg_read(ArmRegister::R3),
                engine.reg_read(ArmRegister::R4),
                engine.reg_read(ArmRegister::R5),
                engine.reg_read(ArmRegister::R6),
                engine.reg_read(ArmRegister::R7),
                engine.reg_read(ArmRegister::R8),
            ),
            format!(
                "SB: {:#x} SL: {:#x} FP: {:#x} IP: {:#x} SP: {:#x} LR: {:#x} PC: {:#x}",
                engine.reg_read(ArmRegister::SB),
                engine.reg_read(ArmRegister::SL),
                engine.reg_read(ArmRegister::FP),
                engine.reg_read(ArmRegister::IP),
                engine.reg_read(ArmRegister::SP),
                engine.reg_read(ArmRegister::LR),
                engine.reg_read(ArmRegister::PC),
            ),
            format!("CPSR: {:032b}\n", engine.reg_read(ArmRegister::Cpsr)),
        ]
        .join("\n")
    }

    fn is_code_address(address: u32, image_base: u32) -> bool {
        // TODO image size temp

        (address % 2 == 1 && (image_base..image_base + 0x100000).contains(&address))
            || (FUNCTIONS_BASE..FUNCTIONS_BASE + FUNCTIONS_SIZE as u32).contains(&address)
    }

    fn dump_regs(&self) -> String {
        let inner = self.inner.lock();

        Self::dump_regs_inner(&*inner.engine)
    }

    fn format_callstack_address(address: u32, image_base: u32) -> String {
        let description = if (image_base..image_base + 0x100000).contains(&address) {
            format!("<Base>+{:#x}", address - image_base)
        } else if (FUNCTIONS_BASE..FUNCTIONS_BASE + FUNCTIONS_SIZE as u32).contains(&address) {
            "<Native function>".to_owned()
        } else {
            "<Unknown>".to_owned()
        };

        format!("{address:#x}: {description}\n")
    }

    fn dump_call_stack(&self, image_base: u32) -> Result<String> {
        let mut inner = self.inner.lock();

        let sp = inner.engine.reg_read(ArmRegister::SP);
        let pc = inner.engine.reg_read(ArmRegister::PC);
        let lr = inner.engine.reg_read(ArmRegister::LR);

        let mut call_stack = Self::format_callstack_address(pc, image_base);
        if lr != RUN_FUNCTION_LR && lr != 0 {
            call_stack += &Self::format_callstack_address(lr - 5, image_base);
        }

        for i in 0..128 {
            let address = sp + (i * 4);
            if !inner.engine.is_mapped(address, size_of::<u32>()) {
                break;
            }

            let mut value = [0; size_of::<u32>()];
            inner.engine.mem_read(address, size_of::<u32>(), &mut value)?;
            let value_u32 = u32::from_le_bytes(value);

            if value_u32 > 5 && Self::is_code_address(value_u32 - 4, image_base) {
                call_stack += &Self::format_callstack_address(value_u32 - 5, image_base);
            }
        }

        Ok(call_stack)
    }

    fn dump_stack(&self) -> Result<String> {
        let mut inner = self.inner.lock();

        let sp = inner.engine.reg_read(ArmRegister::SP);

        let mut result = String::new();
        for i in 0..16 {
            let address = sp + (i * 4);

            if !inner.engine.is_mapped(address, size_of::<u32>()) {
                break;
            }

            let mut value = [0; size_of::<u32>()];
            inner.engine.mem_read(address, size_of::<u32>(), &mut value)?;
            let value_u32 = u32::from_le_bytes(value);

            result += &format!("SP+{:#x}: {value_u32:#x}\n", i * 4);
        }

        Ok(result)
    }
}

impl ByteRead for ArmCore {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> wie_util::Result<usize> {
        let mut inner = self.inner.lock();

        let read = inner.engine.mem_read(address, result.len(), result)?;

        Ok(read)
    }
}

impl ByteWrite for ArmCore {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        let mut inner = self.inner.lock();

        inner.engine.mem_write(address, data)?;

        Ok(())
    }
}

pub trait RunFunctionResult<R> {
    fn get(core: &ArmCore) -> R;
}

impl RunFunctionResult<u32> for u32 {
    fn get(core: &ArmCore) -> u32 {
        core.read_param(0).unwrap()
    }
}

impl RunFunctionResult<()> for () {
    fn get(_: &ArmCore) {}
}

pub struct ThreadContextGuard {
    core: ArmCore,
    thread_id: ThreadId,
}

impl ThreadContextGuard {
    pub fn new(mut core: ArmCore, thread_id: ThreadId) -> Self {
        let context = core.threads.lock().get(&thread_id).unwrap().context.clone();
        core.restore_context(&context);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(debug) = core.debug_inner() {
            debug.on_thread_entered(thread_id);
        }

        Self { core, thread_id }
    }
}

impl Drop for ThreadContextGuard {
    fn drop(&mut self) {
        let context = self.core.save_context();

        self.core.threads.lock().get_mut(&self.thread_id).unwrap().context = context;

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(debug) = self.core.debug_inner() {
            debug.on_thread_exited(self.thread_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{
        pin::pin,
        sync::atomic::{AtomicU32, Ordering},
        task::{Context, Poll, Waker},
    };

    use crate::function::JumpTo;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn saved_thread_contexts_are_accessible_while_the_engine_is_locked() {
        extern crate std;

        let mut core = ArmCore::new(false, None).unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let _thread = core.run_in_thread(|| async { Ok(()) }).unwrap();
        let mut context = core.read_thread_context(1).unwrap();
        context.r0 = 42;

        let mut observer = core.clone();
        let (tx, rx) = crossbeam::channel::bounded(1);
        let engine_guard = core.inner.lock();
        let reader = std::thread::spawn(move || {
            observer.write_thread_context(1, &context);
            tx.send((observer.read_thread_context(1).unwrap().r0, observer.get_thread_ids())).unwrap();
        });
        let result = rx.recv_timeout(std::time::Duration::from_secs(1));
        drop(engine_guard);
        reader.join().unwrap();
        assert_eq!(result.unwrap(), (42, vec![1]));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn scheduler_locking_yields_without_resetting_the_instruction_budget() {
        let mut core = ArmCore::new(false, None).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        core.inner.lock().instructions_remaining = 5;
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        debug.on_thread_entered(1);
        debug.resume(None, Some(vec![2]));

        let observer = core.clone();
        let mut run = pin!(core.run_function::<()>(0x1001, &[]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.inner.lock().instructions_remaining, 5);
        debug.resume(None, None);
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert_eq!(observer.inner.lock().instructions_remaining, 4);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn scheduler_locking_keeps_other_thread_futures_stopped() {
        let mut core = ArmCore::new(false, None).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let mut task = pin!(
            core.run_in_thread(move || async move {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .unwrap()
        );
        let _other = core.run_in_thread(|| async { Ok(()) }).unwrap();
        debug.resume(None, Some(vec![2]));
        let (waker, wake_count) = futures_test::task::new_count_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(wake_count, 0);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        debug.resume(None, None);
        assert!(matches!(task.as_mut().poll(&mut cx), Poll::Ready(Ok(()))));
        assert_eq!(observed.load(Ordering::Relaxed), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn interrupt_stops_a_thread_waiting_in_host_code() {
        extern crate std;

        let mut core = ArmCore::new(false, None).unwrap();
        let engine = DebuggedArm32CpuEngine::new();
        let debug = engine.debug_inner();
        core.inner.lock().engine = Box::new(engine);
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let mut task = Box::pin(
            core.run_in_thread(move || async move {
                loop {
                    calls.fetch_add(1, Ordering::Relaxed);
                    YieldFuture::new().await;
                }
            })
            .unwrap(),
        );
        let _other = core.run_in_thread(|| async { Ok(()) }).unwrap();
        debug.resume(None, None);
        assert!(task.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_pending());

        debug.interrupt();
        let runner = std::thread::spawn(move || {
            assert!(task.as_mut().poll(&mut Context::from_waker(Waker::noop())).is_pending());
            task
        });
        let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
        let mut context = debug.read_registers();
        context.r0 = 123;
        debug.write_registers(&context);
        debug.resume(None, Some(vec![2]));
        let _task = runner.join().unwrap();
        assert!(matches!(
            stopped,
            Ok(crate::engine::DebugStopReason::Signal(crate::engine::DebugSignal::Int, 1))
        ));
        assert_eq!(observed.load(Ordering::Relaxed), 1);
        assert_eq!(core.read_thread_context(1).unwrap().r0, 123);
    }

    async fn test_svc_handler(_core: &mut ArmCore, seen_id: &mut Option<u32>, id: crate::SvcId) -> Result<()> {
        *seen_id = Some(id.0);

        Ok(())
    }

    async fn count_inline_svc(core: &mut ArmCore, calls: &mut Arc<AtomicU32>) -> Result<JumpTo> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(JumpTo(core.read_pc_lr()?.0 | 1))
    }

    async fn call_nested_arm(core: &mut ArmCore, result: &mut Arc<AtomicU32>) -> Result<JumpTo> {
        let pc = core.read_pc_lr()?.0;
        let value = core.run_function(0x10001, &[0]).await?;
        result.store(value, Ordering::Relaxed);
        Ok(JumpTo(pc | 1))
    }

    #[test]
    fn yields_every_ten_thousand_instructions_across_svc() {
        let mut core = ArmCore::new(false, None).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count_inline_svc, &calls).unwrap();
        let mut code = [0x01, 0x30, 0x01, 0xdf].repeat(10_000); // add r0, #1; svc #1
        code.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&code, 0x1000, code.len()).unwrap();

        let observer = core.clone();
        let mut run = pin!(core.run_function::<u32>(0x1001, &[0]));
        let mut cx = Context::from_waker(Waker::noop());
        for completed in [5_000, 10_000] {
            assert!(run.as_mut().poll(&mut cx).is_pending());
            assert_eq!(observer.read_param(0).unwrap(), completed);
            assert_eq!(calls.load(Ordering::Relaxed), completed - 1);
            assert_eq!(observer.save_context().cpsr & 0x3f, 0x3f);
        }
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(10_000))));
        assert_eq!(calls.load(Ordering::Relaxed), 10_000);
    }

    #[test]
    fn nested_arm_calls_share_the_instruction_budget() {
        let mut core = ArmCore::new(false, None).unwrap();
        let result = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, call_nested_arm, &result).unwrap();
        let mut outer = [0xc0, 0x46].repeat(5_000); // nop
        outer.extend_from_slice(&[0x01, 0xdf, 0x70, 0x47]); // svc #1; bx lr
        core.load(&outer, 0x1000, outer.len()).unwrap();
        let mut nested = [0x01, 0x30].repeat(5_000); // add r0, #1
        nested.extend_from_slice(&[0x70, 0x47]); // bx lr
        core.load(&nested, 0x10000, nested.len()).unwrap();

        let observer = core.clone();
        let mut run = pin!(core.run_function::<u32>(0x1001, &[17]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.read_param(0).unwrap(), 4_999);
        assert_eq!(result.load(Ordering::Relaxed), 0);
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(17))));
        assert_eq!(result.load(Ordering::Relaxed), 5_000);
    }

    #[test]
    fn returning_arm_calls_do_not_reset_the_instruction_budget() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        let completed = Arc::new(AtomicU32::new(0));
        let observed = completed.clone();
        let mut run = pin!(async move {
            for _ in 0..10_001 {
                core.run_function::<()>(0x1001, &[]).await.unwrap();
                completed.fetch_add(1, Ordering::Relaxed);
            }
        });
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observed.load(Ordering::Relaxed), 9_999);
        assert!(run.as_mut().poll(&mut cx).is_ready());
        assert_eq!(observed.load(Ordering::Relaxed), 10_001);
    }

    #[test]
    fn test_thumb_svc_stub_dispatch() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x1000, 0x1000).unwrap();

        let mut context = core.save_context();
        context.sp = 0x2000;
        core.restore_context(&context);

        core.register_svc_handler(1, test_svc_handler, &None).unwrap();
        let first = core.make_svc_stub(1, 0u32).unwrap();
        let second = core.make_svc_stub(1, 1u32).unwrap();
        assert_eq!(first, FUNCTIONS_BASE + 1);
        assert_eq!(second, FUNCTIONS_BASE + SVC_STUB_SIZE + 1);

        let result = {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::Cpsr, 0x3f);
            inner.engine.reg_write(ArmRegister::PC, second);
            inner.engine.reg_write(ArmRegister::LR, RUN_FUNCTION_LR);
            inner.engine.run(RUN_FUNCTION_LR, 10).unwrap()
        };

        assert_eq!(result.instructions_executed, 5);
        match result.stop_reason {
            EngineStopReason::Svc { category, lr, spsr } => {
                assert_eq!(category, 1);
                assert_eq!(lr, FUNCTIONS_BASE + SVC_STUB_SIZE + 10);
                assert_ne!(spsr & 0x20, 0);
            }
            EngineStopReason::End => panic!("expected SVC, got end"),
            EngineStopReason::Yield => panic!("expected SVC, got yield"),
        }
    }
}
