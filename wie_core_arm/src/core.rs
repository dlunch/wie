use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, sync::Arc, vec::Vec};
use core::mem::size_of;

use spin::Mutex;

use wie_util::{ByteRead, ByteWrite, Result, read_generic};

use crate::{
    ThreadId,
    context::ArmCoreContext,
    engine::{ArmEngine, ArmRegister, MemoryPermission},
    function::{EmulatedFunction, RegisteredFunction, RegisteredFunctionHolder, ResultWriter},
    thread::ThreadState,
    thread_wrapper::ArmCoreThreadWrapper,
};

const FUNCTIONS_BASE: u32 = 0x71000000;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const HEAP_SIZE: u32 = 0x1000000; // 16mb

struct ArmCoreInner {
    engine: Box<dyn ArmEngine>,
    last_thread_id: ThreadId,
    threads: BTreeMap<ThreadId, ThreadState>,
    functions: BTreeMap<u32, Arc<Box<dyn RegisteredFunction>>>,
}

#[derive(Clone)]
pub struct ArmCore {
    inner: Arc<Mutex<ArmCoreInner>>, // TODO can we change it to another lock like async-lock?
}

impl ArmCore {
    pub fn new(enable_gdbserver: bool) -> Result<Self> {
        let mut engine = Box::new(crate::engine::Arm32CpuEngine::new());

        engine.mem_map(FUNCTIONS_BASE, 0x1000, MemoryPermission::ReadExecute);

        let inner = ArmCoreInner {
            engine,
            last_thread_id: 0,
            threads: BTreeMap::new(),
            functions: BTreeMap::new(),
        };

        let result = Self {
            inner: Arc::new(Mutex::new(inner)),
        };

        if enable_gdbserver {
            #[cfg(not(target_arch = "wasm32"))]
            crate::gdb::GdbTarget::start(result.clone());
            #[cfg(target_arch = "wasm32")]
            panic!("GDB server is not supported on wasm32"); // TODO graceful error handling
        }

        Ok(result)
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

        ArmCoreThreadWrapper::new(self.clone(), thread_id, entry)
    }

    pub fn delete_thread_context(&self, thread_id: ThreadId) {
        tracing::info!("Terminate thread: {thread_id}");

        // we should exit inner lock first to run cleanup on thread state drop
        let _ = {
            let mut inner = self.inner.lock();
            inner.threads.remove(&thread_id)
        };
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
        }

        loop {
            let pc = {
                let mut inner = self.inner.lock();
                inner.engine.run(RUN_FUNCTION_LR, FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000, 1000)?
            };

            if pc == RUN_FUNCTION_LR {
                break;
            }

            if (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000).contains(&pc) {
                let mut self1 = self.clone();

                let function = {
                    let inner = self.inner.lock();

                    inner.functions.get(&pc).unwrap().clone()
                };

                function.call(&mut self1).await?;
            }
        }

        let result = R::get(self);
        self.restore_context(&previous_context);

        Ok(result)
    }

    pub fn register_function<F, C, R, P>(&mut self, function: F, context: &C) -> Result<u32>
    where
        F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
        C: Clone + 'static + Sync + Send,
        R: ResultWriter<R> + 'static + Sync + Send,
        P: 'static + Sync + Send,
    {
        let mut inner = self.inner.lock();

        let count = inner.functions.len();

        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE as u64 + (count * 2) as u64;

        inner.engine.mem_write(address as u32, &bytes)?;

        let callback = RegisteredFunctionHolder::new(function, context);

        inner.functions.insert(address as u32, Arc::new(Box::new(callback)));

        tracing::trace!("Register function at {:#x}", address);

        Ok(address as u32 + 1)
    }

    pub fn map(&mut self, address: u32, size: u32) -> Result<()> {
        tracing::trace!("Map address: {:#x}, size: {:#x}", address, size);

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

        address % 2 == 1 && ((image_base..image_base + 0x100000).contains(&address) || (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x10000).contains(&address))
    }

    fn dump_regs(&self) -> String {
        let inner = self.inner.lock();

        Self::dump_regs_inner(&*inner.engine)
    }

    fn format_callstack_address(address: u32, image_base: u32) -> String {
        let description = if (image_base..image_base + 0x100000).contains(&address) {
            format!("<Base>+{:#x}", address - image_base)
        } else if (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x10000).contains(&address) {
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

            result += &format!("SP+{:#x}: {:#x}\n", i * 4, value_u32);
        }

        Ok(result)
    }
}

impl ByteRead for ArmCore {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> wie_util::Result<usize> {
        let mut inner = self.inner.lock();

        let read = inner.engine.mem_read(address, result.len(), result)?;

        // tracing::trace!("Read address: {:#x}, data: {:02x?}", address, data);

        Ok(read)
    }
}

impl ByteWrite for ArmCore {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        // tracing::trace!("Write address: {:#x}, data: {:02x?}", address, data);
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

        Self { core, thread_id }
    }
}

impl Drop for ThreadContextGuard {
    fn drop(&mut self) {
        let context = self.core.save_context();

        self.core.inner.lock().threads.get_mut(&self.thread_id).unwrap().context = context;
    }
}
