use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, sync::Arc, vec::Vec};
use core::{future::poll_fn, mem::size_of, pin::pin};

use spin::Mutex;

use wie_arm_jit_types::PreparationState;
use wie_backend::{ProfileCallback, YieldFuture};
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use crate::{
    EmulatedFunction, ResultWriter, ThreadId,
    context::ArmCoreContext,
    engine::{Arm32CpuEngine, ArmEngine, ArmRegister, DebugInner, DebuggedArm32CpuEngine, EngineStopReason, MemoryPermission},
    function::{RegisteredFunction, RegisteredFunctionHolder},
    thread::ThreadState,
    thread_wrapper::ArmCoreThreadWrapper,
};

const GLOBAL_DATA_BASE: u32 = 0x7fff0000;
const FUNCTIONS_BASE: u32 = 0x71000000;
const FUNCTIONS_SIZE: usize = 0x10000;
const SVC_STUB_SIZE: u32 = 16;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const HEAP_SIZE: u32 = 0x10000000;
#[cfg(not(target_arch = "wasm32"))]
const INSTRUCTIONS_PER_YIELD: u32 = 10_000;

pub(crate) struct ArmCoreInner {
    pub(crate) engine: Box<dyn ArmEngine>,
    #[cfg(not(target_arch = "wasm32"))]
    instructions_remaining: u32,
    last_thread_id: ThreadId,
    svc_handlers: BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>,
    next_stub_address: u32,
    profile: Option<ProfileCallback>,
    closed: bool,
}

impl Drop for ArmCoreInner {
    fn drop(&mut self) {
        self.engine.shutdown();
        let batch = self.engine.take_profile(true);
        if !batch.is_empty()
            && let Some(callback) = self.profile.as_mut()
        {
            callback(batch);
        }
    }
}

#[derive(Clone)]
pub struct ArmCore {
    pub(crate) inner: Arc<Mutex<ArmCoreInner>>, // TODO can we change it to another lock like async-lock?
    threads: Arc<Mutex<BTreeMap<ThreadId, ThreadState>>>,
}

impl ArmCore {
    pub fn new(enable_gdbserver: bool, profile: Option<ProfileCallback>) -> Result<Self> {
        let mut engine = if enable_gdbserver {
            Box::new(DebuggedArm32CpuEngine::new()) as Box<dyn ArmEngine>
        } else {
            Box::new(Arm32CpuEngine::with_backend())
        };

        engine.set_profiling(profile.is_some());
        engine.mem_map(FUNCTIONS_BASE, FUNCTIONS_SIZE, MemoryPermission::ReadExecute);
        engine.mem_map(GLOBAL_DATA_BASE, 0x4000, MemoryPermission::ReadWriteExecute);

        let inner = ArmCoreInner {
            engine,
            #[cfg(not(target_arch = "wasm32"))]
            instructions_remaining: INSTRUCTIONS_PER_YIELD,
            last_thread_id: 0,
            svc_handlers: BTreeMap::new(),
            next_stub_address: FUNCTIONS_BASE,
            profile,
            closed: false,
        };

        let result = Self {
            inner: Arc::new(Mutex::new(inner)),
            threads: Arc::new(Mutex::new(BTreeMap::new())),
        };

        if enable_gdbserver && let Err(error) = crate::gdb::start(result.clone()) {
            result.shutdown();
            return Err(error);
        }

        Ok(result)
    }

    pub fn shutdown(&self) {
        let handlers = {
            let mut inner = self.inner.lock();
            if !inner.closed {
                inner.closed = true;
                inner.engine.shutdown();
            }
            core::mem::take(&mut inner.svc_handlers)
        };
        self.flush_profile();
        // Handler contexts can own this core and run destructors that reenter it.
        drop(handlers);
    }

    pub fn check_running(&self) -> Result<()> {
        if self.inner.lock().closed {
            return Err(WieError::FatalError("ARM core is shut down".into()));
        }
        Ok(())
    }

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
        inner.engine.record_image(address, data.len());

        Ok(())
    }

    pub async fn prepare_execution(&mut self) -> Result<()> {
        self.check_running()?;
        let preparation = self.inner.lock().engine.begin_preparation()?;
        if let Some(preparation) = preparation {
            let result = preparation.await;
            self.check_running()?;
            self.inner.lock().engine.finish_preparation(result);
        }
        Ok(())
    }

    pub fn is_preparing(&self) -> bool {
        self.inner.lock().engine.preparation_state() != PreparationState::Ready
    }

    pub fn run_in_thread<F, Fut>(&self, entry: F) -> Result<ArmCoreThreadWrapper>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.check_running()?;
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

    fn flush_profile(&self) {
        // Taking the FnMut also serializes reentrant delivery; nested runs leave samples in the engine.
        let Some(mut callback) = self.inner.lock().profile.take() else {
            return;
        };
        loop {
            let mut inner = self.inner.lock();
            let closed = inner.closed;
            let batch = inner.engine.take_profile(closed);
            if batch.is_empty() {
                if !closed {
                    inner.profile = Some(callback);
                }
                return;
            }
            drop(inner);
            callback(batch);
        }
    }

    pub async fn run_function<R>(&mut self, address: u32, params: &[u32]) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        // we don't need to save r0-r3, but to make it simple, we save all registers
        let previous_context = self.save_context();
        let result = self.run_function_inner(address, params).await;
        // Closed cores retain the fault state; recoverable calls restore their caller.
        if !self.inner.lock().closed {
            self.restore_context(&previous_context);
        }
        result
    }

    async fn run_function_inner<R>(&mut self, address: u32, params: &[u32]) -> Result<R>
    where
        R: RunFunctionResult<R>,
    {
        {
            let mut inner = self.inner.lock();
            if inner.closed {
                return Err(WieError::FatalError("ARM core is shut down".into()));
            }

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
            let (result, exhausted) = {
                let mut inner = self.inner.lock();
                if inner.closed {
                    return Err(WieError::FatalError("ARM core is shut down".into()));
                }
                #[cfg(not(target_arch = "wasm32"))]
                let budget = inner.instructions_remaining;
                #[cfg(target_arch = "wasm32")]
                let budget = u32::MAX;
                let result = match inner.engine.run(RUN_FUNCTION_LR, budget) {
                    Ok(result) => result,
                    Err(error) => {
                        inner.closed = true;
                        inner.engine.shutdown();
                        drop(inner);
                        self.shutdown();
                        return Err(error);
                    }
                };
                #[cfg(not(target_arch = "wasm32"))]
                let exhausted = {
                    inner.instructions_remaining -= result.instructions_executed;
                    let exhausted = inner.instructions_remaining == 0;
                    if exhausted {
                        inner.instructions_remaining = INSTRUCTIONS_PER_YIELD;
                    }
                    exhausted
                };
                #[cfg(target_arch = "wasm32")]
                let exhausted = false;
                (result.stop_reason, exhausted)
            };

            let svc_step = if let EngineStopReason::Svc { lr, spsr, .. } = result {
                // Leave exception mode before yielding: thread contexts do not save banked SVC registers.
                {
                    let mut inner = self.inner.lock();
                    inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                    inner.engine.reg_write(ArmRegister::PC, lr);
                }
                // Nested ARM calls remain part of this handler's single step.
                self.debug_inner()
                    .and_then(|debug| debug.begin_svc_step().map(|thread_id| (debug, thread_id)))
            } else {
                None
            };

            let should_yield = exhausted || matches!(result, EngineStopReason::Yield);
            self.flush_profile();

            if should_yield {
                YieldFuture::new().await;
            }
            self.check_running()?;

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
                    let mut call = pin!(function.call(&mut self1));
                    let result = poll_fn(|cx| {
                        self.check_running()?;
                        call.as_mut().poll(cx)
                    })
                    .await;
                    if let Some((debug, thread_id)) = svc_step {
                        debug.end_svc_step(thread_id, result.is_ok());
                    }
                    result?;
                }
            }
        }

        Ok(R::get(self))
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

        if let Some(debug) = self.core.debug_inner() {
            debug.on_thread_exited(self.thread_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use core::{
        pin::pin,
        sync::atomic::{AtomicBool, AtomicU32, Ordering},
        task::{Context, Poll, Waker},
    };

    use crate::function::JumpTo;
    use wie_util::read_null_terminated_string_bytes;

    use super::*;

    #[test]
    fn terminated_string_reads_use_normal_and_debug_engines() {
        for debug in [false, true] {
            let mut core = ArmCore::new(false, None).unwrap();
            if debug {
                core.inner.lock().engine = Box::new(DebuggedArm32CpuEngine::new());
            }
            core.load(&[], 0x10000, 0x10000).unwrap();
            core.write_bytes(0x1fffc, b"end\0").unwrap();
            let reader: &dyn ByteRead = &core;
            assert_eq!(read_null_terminated_string_bytes(reader, 0x1fffc).unwrap(), b"end");
            assert!(read_null_terminated_string_bytes(reader, 0x1ffff).unwrap().is_empty());
            assert!(matches!(
                read_null_terminated_string_bytes(reader, 0),
                Err(WieError::InvalidMemoryAccess(0))
            ));
            core.write_bytes(0x1ffff, b"x").unwrap();
            assert!(matches!(
                read_null_terminated_string_bytes(&core, 0x1fffc),
                Err(WieError::InvalidMemoryAccess(0x20000))
            ));
            core.load(b"page\0", 0x20000, 5).unwrap();
            assert_eq!(read_null_terminated_string_bytes(&core, 0x1fffc).unwrap(), b"endxpage");
            core.load(b"raw\0", 0, 4).unwrap();
            assert!(matches!(
                read_null_terminated_string_bytes(&core, 0),
                Err(WieError::InvalidMemoryAccess(0))
            ));
        }
    }

    #[test]
    fn shutdown_releases_svc_contexts_outside_the_core_lock() {
        struct ContextOwner(ArmCore);

        impl Drop for ContextOwner {
            fn drop(&mut self) {
                assert!(self.0.inner.try_lock().is_some());
                self.0.shutdown();
            }
        }

        async fn handler(_: &mut ArmCore, _: &mut Arc<ContextOwner>) -> Result<()> {
            Ok(())
        }

        let mut core = ArmCore::new(false, None).unwrap();
        let weak = Arc::downgrade(&core.inner);
        let context = Arc::new(ContextOwner(core.clone()));
        let weak_context = Arc::downgrade(&context);
        core.register_svc_handler(1, handler, &context).unwrap();
        drop(context);

        core.shutdown();
        assert!(weak_context.upgrade().is_none());
        drop(core);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn shutdown_blocks_clones_before_call_setup() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap(); // bx lr
        let mut remaining = core.clone();
        let previous = remaining.save_context();

        core.shutdown();
        core.shutdown();
        assert!(futures::executor::block_on(remaining.run_function::<()>(0x1001, &[1, 2, 3, 4, 5])).is_err());
        let after = remaining.save_context();
        assert_eq!(
            (after.r0, after.sp, after.pc, after.cpsr),
            (previous.r0, previous.sp, previous.pc, previous.cpsr)
        );
        assert!(remaining.run_in_thread(|| async { Ok(()) }).is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn shutdown_stops_a_suspended_svc_before_its_handler() {
        let mut core = ArmCore::new(false, None).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, count_inline_svc, &calls).unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x1000, 4).unwrap();
        core.inner.lock().instructions_remaining = 1;
        let observer = core.clone();
        let mut run = pin!(core.run_function::<()>(0x1001, &[]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.save_context().pc, 0x1002);
        assert_eq!(observer.save_context().cpsr & 0x3f, 0x3f);
        observer.shutdown();
        assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Err(_))));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    async fn pending_svc_handler(_: &mut ArmCore, (entered, calls): &mut (Arc<AtomicBool>, Arc<AtomicU32>)) -> Result<u32> {
        entered.store(true, Ordering::Relaxed);
        let mut yielded = false;
        poll_fn(|_| {
            if core::mem::replace(&mut yielded, true) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(42)
    }

    #[test]
    fn shutdown_does_not_resume_an_already_pending_svc_handler() {
        let mut core = ArmCore::new(false, None).unwrap();
        let entered = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, pending_svc_handler, &(entered.clone(), calls.clone()))
            .unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x1000, 4).unwrap();
        let observer = core.clone();
        let mut run = pin!(core.run_function::<()>(0x1001, &[]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert!(entered.load(Ordering::Relaxed));
        assert_eq!(observer.save_context().pc, 0x1002);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        observer.shutdown();
        let before = observer.save_context();
        let result = run.as_mut().poll(&mut cx);
        let after = observer.save_context();
        assert!(matches!(result, Poll::Ready(Err(_))));
        assert_eq!((calls.load(Ordering::Relaxed), after.r0, after.pc), (0, before.r0, before.pc));
    }

    #[test]
    fn shutdown_does_not_resume_an_existing_host_thread() {
        let mut core = ArmCore::new(false, None).unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let calls = Arc::new(AtomicU32::new(0));
        let observed = calls.clone();
        let task = core
            .run_in_thread(move || async move {
                let mut yielded = false;
                poll_fn(|_| {
                    if core::mem::replace(&mut yielded, true) {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .unwrap();
        let mut task = pin!(task);
        let mut cx = Context::from_waker(Waker::noop());
        assert!(task.as_mut().poll(&mut cx).is_pending());
        core.shutdown();
        let result = task.as_mut().poll(&mut cx);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        assert!(matches!(result, Poll::Ready(Err(_))));
    }

    #[test]
    fn shutdown_preserves_fault_context_when_a_nested_thread_is_polled() {
        let mut core = ArmCore::new(false, None).unwrap();
        crate::Allocator::init(&mut core).unwrap();
        let completed = Arc::new(AtomicU32::new(0));
        let entered = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicU32::new(0));
        core.register_svc_handler(1, call_nested_arm, &completed).unwrap();
        core.register_svc_handler(2, pending_svc_handler, &(entered.clone(), calls.clone()))
            .unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x1000, 4).unwrap();
        core.load(&[0x02, 0xdf, 0x70, 0x47], 0x10000, 4).unwrap();
        let mut running = core.clone();
        let mut task = pin!(
            core.run_in_thread(move || async move { running.run_function::<()>(0x1001, &[]).await })
                .unwrap()
        );
        let mut cx = Context::from_waker(Waker::noop());
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert!(entered.load(Ordering::Relaxed));
        assert_eq!(core.read_thread_context(1).unwrap().pc, 0x10002);
        assert_eq!(calls.load(Ordering::Relaxed), 0);

        assert!(futures::executor::block_on(core.run_function::<()>(1, &[99])).is_err());
        assert!(core.check_running().is_err());
        let diagnostic = core.dump_regs();
        assert!(matches!(task.as_mut().poll(&mut cx), Poll::Ready(Err(_))));
        assert_eq!(core.dump_regs(), diagnostic);
        assert_eq!(core.read_thread_context(1).unwrap().pc, 0x10002);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert_eq!(completed.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn shutdown_inside_a_svc_handler_prevents_result_writes() {
        async fn shutdown_handler(core: &mut ArmCore, _: &mut ()) -> Result<u32> {
            core.shutdown();
            Ok(42)
        }

        let mut core = ArmCore::new(false, None).unwrap();
        core.register_svc_handler(1, shutdown_handler, &()).unwrap();
        core.load(&[0x01, 0xdf, 0x70, 0x47], 0x1000, 4).unwrap();
        assert!(futures::executor::block_on(core.run_function::<()>(0x1001, &[17])).is_err());
        let context = core.save_context();
        assert_eq!((context.r0, context.pc), (17, 0x1002));
    }

    #[test]
    fn engine_errors_close_the_shared_core_without_restoring_fault_state() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.load(&[0x70, 0x47], 0x1000, 2).unwrap();
        let mut remaining = core.clone();
        assert!(futures::executor::block_on(core.run_function::<()>(1, &[42])).is_err());
        let fault = core.save_context();
        assert_eq!((fault.r0, fault.pc), (42, 0));
        assert!(futures::executor::block_on(remaining.run_function::<()>(0x1001, &[7])).is_err());
        let after = remaining.save_context();
        assert_eq!((after.r0, after.pc, after.cpsr), (fault.r0, fault.pc, fault.cpsr));
    }

    #[test]
    fn final_owner_drop_flushes_remaining_profile_samples() {
        let samples = Arc::new(Mutex::new(Vec::new()));
        let observed = samples.clone();
        let mut core = ArmCore::new(false, Some(Box::new(move |batch| samples.lock().extend(batch)))).unwrap();
        let mut code = [0xc0, 0x46].repeat(2048); // nop
        code.extend_from_slice(&[0x70, 0x47]);
        core.load(&code, 0x1000, code.len()).unwrap();
        futures::executor::block_on(core.run_function::<()>(0x1001, &[])).unwrap();
        let last = core.clone();
        drop(core);
        assert!(observed.lock().is_empty());
        drop(last);
        let samples = observed.lock();
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|sample| (0x1000..0x2000).contains(&sample.stack[0])));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn profile_callbacks_can_reenter_and_shutdown_outside_engine_locks() {
        for debug in [false, true] {
            let holder = Arc::new(Mutex::new(None::<ArmCore>));
            let callback_core = holder.clone();
            let callbacks = Arc::new(AtomicU32::new(0));
            let observed = callbacks.clone();
            let mut core = ArmCore::new(
                false,
                Some(Box::new(move |batch| {
                    assert!(!batch.is_empty());
                    let mut core = callback_core.lock().as_ref().unwrap().clone();
                    assert!(core.inner.try_lock().is_some());
                    core.save_context(); // Also acquires the debug CPU lock.
                    if callbacks.fetch_add(1, Ordering::Relaxed) == 0 {
                        assert!(batch.iter().all(|sample| sample.stack[0] == 0x1000));
                        assert!(batch.iter().map(|sample| sample.count).sum::<u64>() >= 1000);
                        futures::executor::block_on(core.run_function::<()>(0x10001, &[])).unwrap();
                        core.shutdown();
                        core.shutdown();
                    } else {
                        assert!(batch.iter().all(|sample| (0x10000..=0x11000).contains(&sample.stack[0])));
                    }
                })),
            )
            .unwrap();
            if debug {
                let mut engine = DebuggedArm32CpuEngine::new();
                engine.set_profiling(true);
                engine.debug_inner().resume(Vec::new(), None);
                core.inner.lock().engine = Box::new(engine);
            }
            core.load(&[0xfe, 0xe7], 0x1000, 2).unwrap(); // b .
            let mut nested = [0xc0, 0x46].repeat(2048);
            nested.extend_from_slice(&[0x70, 0x47]);
            core.load(&nested, 0x10000, nested.len()).unwrap();
            *holder.lock() = Some(core.clone());
            {
                let mut inner = core.inner.lock();
                inner.engine.reg_write(ArmRegister::Cpsr, 0x3f);
                inner.engine.reg_write(ArmRegister::PC, 0x1000);
                // A flush needs 1000 retired samples; each interval is at most 1152 instructions.
                let count = 1152 * 1000;
                let result = inner.engine.run(RUN_FUNCTION_LR, count).unwrap();
                assert!(matches!(result.stop_reason, EngineStopReason::Yield));
                assert_eq!(result.instructions_executed, count);
                inner.instructions_remaining = 0;
            }
            assert_eq!(observed.load(Ordering::Relaxed), 0);
            let mut run = pin!(core.run_function::<()>(0x1001, &[]));
            let mut cx = Context::from_waker(Waker::noop());
            assert!(run.as_mut().poll(&mut cx).is_pending());
            assert_eq!(observed.load(Ordering::Relaxed), 2);
            let finished = run.as_mut().poll(&mut cx);
            let _owner = holder.lock().take();
            assert!(matches!(finished, Poll::Ready(Err(_))));
            assert_eq!(observed.load(Ordering::Relaxed), 2);
        }
    }

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
        debug.resume(Vec::new(), Some(vec![2]));

        let observer = core.clone();
        let mut run = pin!(core.run_function::<()>(0x1001, &[]));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(run.as_mut().poll(&mut cx).is_pending());
        assert_eq!(observer.save_context().pc, 0x1000);
        assert_eq!(observer.inner.lock().instructions_remaining, 5);
        debug.resume(Vec::new(), None);
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
        debug.resume(Vec::new(), Some(vec![2]));
        let (waker, wake_count) = futures_test::task::new_count_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(wake_count, 0);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        debug.resume(Vec::new(), None);
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
        debug.resume(Vec::new(), None);
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
        debug.resume(Vec::new(), Some(vec![2]));
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn stepping_svc_waits_for_nested_handlers_and_preserves_breakpoints() {
        extern crate std;

        use crate::engine::{DebugBreakpointKind, DebugStopReason};

        for breakpoint in [false, true] {
            let mut core = ArmCore::new(false, None).unwrap();
            let engine = DebuggedArm32CpuEngine::new();
            let debug = engine.debug_inner();
            core.inner.lock().engine = Box::new(engine);
            crate::Allocator::init(&mut core).unwrap();
            let result = Arc::new(AtomicU32::new(0));
            let calls = Arc::new(AtomicU32::new(0));
            core.register_svc_handler(1, call_nested_arm, &result).unwrap();
            core.register_svc_handler(2, count_inline_svc, &calls).unwrap();
            core.load(&[0x01, 0xdf, 0x01, 0x30, 0x70, 0x47], 0x1000, 6).unwrap(); // svc #1; add r0, #1; bx lr
            core.load(&[0x02, 0xdf, 0x01, 0x30, 0x70, 0x47], 0x10000, 6).unwrap(); // svc #2; add r0, #1; bx lr
            if breakpoint {
                debug.add_breakpoint(0x10000, DebugBreakpointKind::Thumb16).unwrap();
            }
            let mut running = core.clone();
            let task = core
                .run_in_thread(move || async move { running.run_function::<()>(0x1001, &[0]).await })
                .unwrap();
            debug.resume(vec![1], None);
            let (tx, rx) = crossbeam::channel::bounded(1);
            let runner = std::thread::spawn(move || tx.send(futures::executor::block_on(task)).unwrap());
            let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
            let context = debug.read_registers();
            let completed = (result.load(Ordering::Relaxed), calls.load(Ordering::Relaxed));
            let nested_stop = if breakpoint {
                debug.resume(vec![1], None);
                let stopped = debug.recv_stop_event_timeout(std::time::Duration::from_secs(1));
                Some((stopped, debug.read_registers(), calls.load(Ordering::Relaxed)))
            } else {
                None
            };
            debug.resume(Vec::new(), None);
            let finished = rx.recv_timeout(std::time::Duration::from_secs(1));
            debug.resume(Vec::new(), None);
            runner.join().unwrap();
            finished.unwrap().unwrap();

            if let Some((nested_stop, nested_context, nested_calls)) = nested_stop {
                assert!(matches!(stopped, Ok(DebugStopReason::SwBreak(1))));
                assert_eq!(context.pc, 0x10000);
                assert_eq!(completed, (0, 0));
                assert!(matches!(nested_stop, Ok(DebugStopReason::DoneStep(1))));
                assert_eq!(nested_context.pc, 0x10002);
                assert_eq!(nested_calls, 1);
            } else {
                assert!(matches!(stopped, Ok(DebugStopReason::DoneStep(1))));
                assert_eq!(context.pc, 0x1002);
                assert_eq!(context.r0, 0);
                assert_eq!(completed, (1, 1));
            }
            assert_eq!(result.load(Ordering::Relaxed), 1);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
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
    fn recoverable_arm_call_failures_restore_caller_registers_and_stack() {
        async fn throwing_handler(core: &mut ArmCore, _: &mut ()) -> Result<()> {
            core.write_return_value(&[99, 98])?;
            Err(WieError::JavaException(0x1234))
        }

        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x2000, 0x1000).unwrap();
        core.register_svc_handler(1, throwing_handler, &()).unwrap();
        let target = core.make_svc_stub(1, 0u32).unwrap();
        let mut context = core.save_context();
        context.r0 = 42;
        context.r1 = 43;
        context.lr = 0x4001;

        for sp in [0x3000, 0x2000_0004] {
            context.sp = sp;
            core.restore_context(&context);
            let registers = core.dump_regs();
            let result = futures::executor::block_on(core.run_function::<()>(target, &[1, 2, 3, 4, 5]));
            if sp == 0x3000 {
                assert!(matches!(result, Err(WieError::JavaException(0x1234))));
            } else {
                assert!(matches!(result, Err(WieError::InvalidMemoryAccess(_))));
            }
            assert!(core.check_running().is_ok());
            assert_eq!(core.dump_regs(), registers);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn ready_arm_threads_yield_and_allow_host_tasks_to_run() {
        use test_utils::{TestClock, TestPlatform};
        use wie_backend::{DefaultTaskRunner, System};

        let clock = TestClock::new();
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "test", "test", DefaultTaskRunner);
        let mut core = ArmCore::new(false, None).unwrap();
        crate::Allocator::init(&mut core).unwrap();
        core.load(&[0x01, 0x30, 0xfd, 0xe7], 0x1000, 4).unwrap(); // add r0, #1; b 0x1000
        let polls = Arc::new(AtomicU32::new(0));
        for _ in 0..2 {
            let mut running = core.clone();
            let task = core
                .run_in_thread(move || async move { running.run_function::<()>(0x1001, &[0]).await })
                .unwrap();
            let clock = clock.clone();
            let polls = polls.clone();
            system.spawn(move || async move {
                let mut task = pin!(task);
                poll_fn(|cx| {
                    let result = task.as_mut().poll(cx);
                    assert!(result.is_pending());
                    polls.fetch_add(1, Ordering::Relaxed);
                    // Limit the executor to one step.
                    clock.advance(9);
                    result
                })
                .await
            });
        }
        let host_calls = Arc::new(AtomicU32::new(0));
        let observed = host_calls.clone();
        system.spawn(move || async move {
            host_calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });

        system.tick().unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 2);
        assert_eq!(observed.load(Ordering::Relaxed), 1);
        let first = [1, 2].map(|thread| core.read_thread_context(thread).unwrap().r0);
        assert_eq!(first, [5_000, 5_000]);
        assert_eq!(core.inner.lock().instructions_remaining, 10_000);
        system.tick().unwrap();
        assert_eq!(polls.load(Ordering::Relaxed), 4);
        for (thread, previous) in [1, 2].into_iter().zip(first) {
            assert_eq!(core.read_thread_context(thread).unwrap().r0 - previous, 5_000);
        }
        system.shutdown();
        assert!(core.get_thread_ids().is_empty());
        core.shutdown();
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
