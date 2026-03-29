use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::mem::{size_of, take};

use spin::Mutex;

use wie_util::{ByteRead, ByteWrite, Result, read_generic};

use crate::{
    ThreadId,
    context::ArmCoreContext,
    engine::{ArmEngine, ArmRegister, EngineRunResult, MemoryPermission},
    function::{EmulatedFunction, RegisteredFunction, RegisteredFunctionHolder, ResultWriter},
    thread::ThreadState,
    thread_wrapper::ArmCoreThreadWrapper,
};

const GLOBAL_DATA_BASE: u32 = 0x7fff0000;
const SVC_FUNCTIONS_BASE: u32 = 0x71001000;
const SVC_FUNCTIONS_SIZE: u32 = 0x10000;
const SVC_STUB_SIZE: u32 = 16;
const ARM_LDR_R12_PC_PLUS_4: u32 = 0xe59fc004;
const ARM_SVC_BASE: u32 = 0xef000000;
const ARM_BX_LR: u32 = 0xe12fff1e;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const HEAP_SIZE: u32 = 0x10000000; // 256 MB

pub(crate) struct ArmCoreInner {
    pub(crate) engine: Box<dyn ArmEngine>,
    last_thread_id: ThreadId,
    threads: BTreeMap<ThreadId, ThreadState>,
    svc_functions: BTreeMap<(u32, u32), Arc<Box<dyn RegisteredFunction>>>,
    next_svc_function_address: u32,
}

#[derive(Clone)]
pub struct ArmCoreState {
    engine: crate::engine::ArmEngineState,
    last_thread_id: ThreadId,
    threads: Vec<ThreadStateSnapshot>,
    svc_function_keys: BTreeSet<(u32, u32)>,
    next_svc_function_address: u32,
}

#[derive(Clone)]
struct ThreadStateSnapshot {
    thread_id: ThreadId,
    context: ArmCoreContext,
    stack_base: usize,
    stack_size: usize,
}

#[derive(Clone)]
pub struct ArmCore {
    pub(crate) inner: Arc<Mutex<ArmCoreInner>>, // TODO can we change it to another lock like async-lock?
}

impl ArmCore {
    pub fn new(enable_gdbserver: bool) -> Result<Self> {
        let mut engine = if enable_gdbserver {
            #[cfg(not(target_arch = "wasm32"))]
            let engine = Box::new(crate::engine::DebuggedArm32CpuEngine::new()) as Box<dyn ArmEngine>;
            #[cfg(target_arch = "wasm32")]
            let engine = Box::new(crate::engine::Arm32CpuEngine::new());

            engine
        } else {
            Box::new(crate::engine::Arm32CpuEngine::new())
        };

        engine.mem_map(SVC_FUNCTIONS_BASE, SVC_FUNCTIONS_SIZE as usize, MemoryPermission::ReadExecute);
        engine.mem_map(GLOBAL_DATA_BASE, 0x4000, MemoryPermission::ReadWriteExecute);

        let inner = ArmCoreInner {
            engine,
            last_thread_id: 0,
            threads: BTreeMap::new(),
            svc_functions: BTreeMap::new(),
            next_svc_function_address: SVC_FUNCTIONS_BASE,
        };

        let result = Self {
            inner: Arc::new(Mutex::new(inner)),
        };

        if enable_gdbserver {
            #[cfg(not(target_arch = "wasm32"))]
            crate::gdb::GdbTarget::start(result.clone())?;
            #[cfg(target_arch = "wasm32")]
            panic!("GDB server is not supported on wasm32");
        }

        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn debug_inner(&self) -> Option<Arc<crate::engine::DebugInner>> {
        let inner = self.inner.lock();

        inner
            .engine
            .as_any()
            .downcast_ref::<crate::engine::DebuggedArm32CpuEngine>()
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
            inner.threads.insert(thread_id, state);

            thread_id
        };

        tracing::info!("Create thread: {thread_id}");

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(debug) = self.debug_inner() {
                debug.on_thread_created(thread_id);
            }
        }

        ArmCoreThreadWrapper::new(self.clone(), thread_id, entry)
    }

    pub fn delete_thread_context(&self, thread_id: ThreadId) {
        tracing::info!("Terminate thread: {thread_id}");

        // we should exit inner lock first to run cleanup on thread state drop
        let _thread_state = {
            let mut inner = self.inner.lock();
            inner.threads.remove(&thread_id)
        };

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(debug) = self.debug_inner() {
            debug.on_thread_deleted(thread_id);
        }
    }

    pub fn enter_thread_context(&self, thread_id: ThreadId) -> ThreadContextGuard {
        ThreadContextGuard::new(self.clone(), thread_id)
    }

    pub fn read_thread_context(&self, thread_id: ThreadId) -> Result<ArmCoreContext> {
        let inner = self.inner.lock();

        let context = inner.threads.get(&thread_id).unwrap().context.clone();

        Ok(context)
    }

    pub fn write_thread_context(&mut self, thread_id: ThreadId, context: &ArmCoreContext) {
        let mut inner = self.inner.lock();

        inner.threads.get_mut(&thread_id).unwrap().context = context.clone();
    }

    pub fn get_thread_ids(&self) -> Vec<ThreadId> {
        let inner = self.inner.lock();

        inner.threads.keys().cloned().collect()
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

            let mut cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
            cpsr = (cpsr & !0x1f) | 0x10;
            cpsr = if address & 1 == 1 { cpsr | 0x20 } else { cpsr & !0x20 };
            inner.engine.reg_write(ArmRegister::Cpsr, cpsr);
        }

        loop {
            let result = {
                let mut inner = self.inner.lock();
                inner.engine.run(RUN_FUNCTION_LR, 1000)?
            };

            match result {
                EngineRunResult::ReachedEnd { pc } => {
                    debug_assert_eq!(pc, RUN_FUNCTION_LR);
                    break;
                }
                EngineRunResult::CountExpired { .. } => continue,
                EngineRunResult::Svc {
                    immediate, r12, lr, spsr, ..
                } => {
                    let mut self1 = self.clone();

                    let function = {
                        let mut inner = self.inner.lock();
                        inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                        inner.svc_functions.get(&(immediate, r12)).unwrap().clone()
                    };

                    function.call(&mut self1, lr).await?;
                }
            }
        }

        let result = R::get(self);
        self.restore_context(&previous_context);

        Ok(result)
    }

    pub fn register_svc_function<F, C, R, P>(&mut self, immediate: u32, function_id: u32, function: F, context: &C) -> Result<u32>
    where
        F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
        C: Clone + 'static + Sync + Send,
        R: ResultWriter<R> + 'static + Sync + Send,
        P: 'static + Sync + Send,
    {
        let mut inner = self.inner.lock();

        if inner.svc_functions.contains_key(&(immediate, function_id)) {
            return Err(wie_util::WieError::FatalError(format!(
                "Duplicate SVC function {immediate}:{function_id}"
            )));
        }

        let address = inner.next_svc_function_address;
        if address + SVC_STUB_SIZE > SVC_FUNCTIONS_BASE + SVC_FUNCTIONS_SIZE {
            return Err(wie_util::WieError::FatalError("SVC stub range exhausted".into()));
        }

        let stub = svc_stub(immediate, function_id);
        inner.engine.mem_write(address, &stub)?;
        inner.next_svc_function_address += SVC_STUB_SIZE;

        let callback = RegisteredFunctionHolder::new(function, context);
        inner.svc_functions.insert((immediate, function_id), Arc::new(Box::new(callback)));

        tracing::trace!("Register svc function #{function_id} at {address:#x}");

        Ok(address)
    }

    pub fn register_svc_handler<F, C, R, P>(&mut self, immediate: u32, function_id: u32, function: F, context: &C) -> Result<()>
    where
        F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
        C: Clone + 'static + Sync + Send,
        R: ResultWriter<R> + 'static + Sync + Send,
        P: 'static + Sync + Send,
    {
        let mut inner = self.inner.lock();

        if inner.svc_functions.contains_key(&(immediate, function_id)) {
            return Err(wie_util::WieError::FatalError(format!(
                "Duplicate SVC function {immediate}:{function_id}"
            )));
        }

        let callback = RegisteredFunctionHolder::new(function, context);
        inner.svc_functions.insert((immediate, function_id), Arc::new(Box::new(callback)));

        tracing::trace!("Register svc handler #{function_id} for svc {immediate}");

        Ok(())
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

    pub fn save_state(&self) -> ArmCoreState {
        let inner = self.inner.lock();

        ArmCoreState {
            engine: inner.engine.save_state(),
            last_thread_id: inner.last_thread_id,
            threads: inner
                .threads
                .iter()
                .map(|(&thread_id, thread)| ThreadStateSnapshot {
                    thread_id,
                    context: thread.context.clone(),
                    stack_base: thread.stack_base,
                    stack_size: thread.stack_size,
                })
                .collect(),
            svc_function_keys: inner.svc_functions.keys().copied().collect(),
            next_svc_function_address: inner.next_svc_function_address,
        }
    }

    pub fn restore_state(&mut self, state: &ArmCoreState) -> Result<()> {
        let old_thread_ids = self.get_thread_ids();
        let old_threads = {
            let mut inner = self.inner.lock();
            take(&mut inner.threads)
        };
        drop(old_threads);

        {
            let mut inner = self.inner.lock();
            inner.engine.restore_state(&state.engine)?;
            inner.last_thread_id = state.last_thread_id;
            if state.svc_function_keys.iter().any(|key| !inner.svc_functions.contains_key(key)) {
                return Err(wie_util::WieError::FatalError("Missing SVC handler for restored state".into()));
            }
            inner.svc_functions.retain(|key, _| state.svc_function_keys.contains(key));
            inner.next_svc_function_address = state.next_svc_function_address;
            inner.threads = state
                .threads
                .iter()
                .map(|thread| {
                    (
                        thread.thread_id,
                        ThreadState::from_parts(self.clone(), thread.context.clone(), thread.stack_base, thread.stack_size),
                    )
                })
                .collect();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(debug) = self.debug_inner() {
            for thread_id in old_thread_ids {
                debug.on_thread_deleted(thread_id);
            }
            for thread in state.threads.iter() {
                debug.on_thread_created(thread.thread_id);
            }
        }

        Ok(())
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

        (SVC_FUNCTIONS_BASE..SVC_FUNCTIONS_BASE + SVC_FUNCTIONS_SIZE).contains(&address)
            || (address % 2 == 1 && (image_base..image_base + 0x100000).contains(&address))
    }

    fn dump_regs(&self) -> String {
        let inner = self.inner.lock();

        Self::dump_regs_inner(&*inner.engine)
    }

    fn format_callstack_address(address: u32, image_base: u32) -> String {
        let description = if (image_base..image_base + 0x100000).contains(&address) {
            format!("<Base>+{:#x}", address - image_base)
        } else if (SVC_FUNCTIONS_BASE..SVC_FUNCTIONS_BASE + SVC_FUNCTIONS_SIZE).contains(&address) {
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

fn svc_stub(immediate: u32, function_id: u32) -> [u8; SVC_STUB_SIZE as usize] {
    bytemuck::cast([ARM_LDR_R12_PC_PLUS_4, ARM_SVC_BASE | (immediate & 0x00ff_ffff), ARM_BX_LR, function_id])
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
        let context = core.inner.lock().threads.get(&thread_id).unwrap().context.clone(); // TODO we might not need clone
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

        let mut inner = self.core.inner.lock();
        inner.threads.get_mut(&self.thread_id).unwrap().context = context;
        drop(inner);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(debug) = self.core.debug_inner() {
            debug.on_thread_exited(self.thread_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use super::{ArmCore, SVC_FUNCTIONS_BASE, SVC_STUB_SIZE};

    async fn increment(_core: &mut ArmCore, _context: &mut (), value: u32) -> Result<u32> {
        Ok(value + 1)
    }

    #[futures_test::test]
    async fn test_svc_alloc_roundtrip() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        let address = core.register_svc_function(1, 1, increment, &())?;

        let result = core.run_function::<u32>(address, &[41]).await?;

        assert_eq!(result, 42);

        Ok(())
    }

    async fn double(_core: &mut ArmCore, _context: &mut (), value: u32) -> Result<u32> {
        Ok(value * 2)
    }

    #[futures_test::test]
    async fn test_svc_dispatch_table_lookup() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        let increment_address = core.register_svc_function(1, 1, increment, &())?;
        let double_address = core.register_svc_function(1, 2, double, &())?;

        assert_eq!(core.run_function::<u32>(increment_address, &[20]).await?, 21);
        assert_eq!(core.run_function::<u32>(double_address, &[20]).await?, 40);

        Ok(())
    }

    #[test]
    fn test_svc_stub_allocator_alignment() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        let first = core.register_svc_function(1, 1, increment, &())?;
        let second = core.register_svc_function(1, 2, increment, &())?;

        assert_eq!(first, SVC_FUNCTIONS_BASE);
        assert_eq!(second, SVC_FUNCTIONS_BASE + SVC_STUB_SIZE);
        assert_eq!(first % 4, 0);
        assert_eq!(second % 4, 0);

        Ok(())
    }

    #[futures_test::test]
    async fn test_save_restore_svc_function_table() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        let address = core.register_svc_function(1, 1, increment, &())?;
        let state = core.save_state();

        assert_eq!(core.run_function::<u32>(address, &[41]).await?, 42);

        core.restore_state(&state)?;

        assert_eq!(core.run_function::<u32>(address, &[41]).await?, 42);

        Ok(())
    }

    #[test]
    fn test_save_restore_svc_allocator_cursor() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.register_svc_function(1, 1, increment, &())?;
        let state = core.save_state();
        let expected = core.register_svc_function(1, 2, increment, &())?;

        core.restore_state(&state)?;

        let restored = core.register_svc_function(1, 2, increment, &())?;
        assert_eq!(restored, expected);

        Ok(())
    }
}
