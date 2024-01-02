use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, rc::Rc, string::String, vec::Vec};
use core::{cell::RefCell, fmt::Debug, mem::size_of};

use wie_backend::{AsyncCallable, SystemHandle};
use wie_base::util::{read_generic, round_up, ByteRead, ByteWrite};

use crate::{
    context::ArmCoreContext,
    engine::{ArmEngine, ArmEngineResult, ArmRegister, MemoryPermission},
    function::{EmulatedFunction, RegisteredFunction, RegisteredFunctionHolder, ResultWriter},
    future::SpawnFuture,
};

const FUNCTIONS_BASE: u32 = 0x71000000;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const PEB_BASE: u32 = 0x7ff00000;

struct ArmCoreInner {
    engine: Box<dyn ArmEngine>,
    system: SystemHandle,
    functions: BTreeMap<u32, Rc<Box<dyn RegisteredFunction>>>,
    functions_count: usize,
}

#[derive(Clone)]
pub struct ArmCore {
    inner: Rc<RefCell<ArmCoreInner>>,
}

impl ArmCore {
    pub fn new(system: SystemHandle) -> ArmEngineResult<Self> {
        #[cfg(any(target_arch = "wasm32", target_os = "linux"))]
        let mut engine = Box::new(crate::engine::Armv4tEmuEngine::new());
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "linux")))]
        let mut engine = Box::new(crate::engine::UnicornEngine::new());

        engine.mem_map(FUNCTIONS_BASE, 0x1000, MemoryPermission::ReadExecute);
        engine.reg_write(ArmRegister::Cpsr, 0x10); // USR32

        let inner = ArmCoreInner {
            engine,
            system,
            functions: BTreeMap::new(),
            functions_count: 0,
        };

        Ok(Self {
            inner: Rc::new(RefCell::new(inner)),
        })
    }

    pub fn load(&mut self, data: &[u8], address: u32, map_size: usize) -> ArmEngineResult<()> {
        let mut inner = self.inner.borrow_mut();

        inner
            .engine
            .mem_map(address, round_up(map_size, 0x1000), MemoryPermission::ReadWriteExecute);
        inner.engine.mem_write(address, data)?;

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
    async fn run_some(&mut self) -> ArmEngineResult<()> {
        let mut inner = self.inner.borrow_mut();

        inner.engine.run(RUN_FUNCTION_LR, FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000, 1000)?;

        let cur_pc = inner.engine.reg_read(ArmRegister::PC);

        if (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000).contains(&cur_pc) {
            let mut self1 = self.clone();
            let mut system_clone = inner.system.clone();

            let function = inner.functions.get(&cur_pc).unwrap().clone();

            drop(inner);

            function.call(&mut self1, &mut system_clone).await?;
        }

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop RefMut https://github.com/rust-lang/rust-clippy/issues/6353
    pub async fn run_function<R>(&mut self, address: u32, params: &[u32]) -> ArmEngineResult<R>
    where
        R: RunFunctionResult<R>,
    {
        let previous_context = self.save_context(); // do we have to save context?
        let mut inner = self.inner.borrow_mut();

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
                let sp = inner.engine.reg_read(ArmRegister::SP) - 4;

                inner.engine.mem_write(sp, &param.to_le_bytes())?;
                inner.engine.reg_write(ArmRegister::SP, sp);
            }
        }

        inner.engine.reg_write(ArmRegister::PC, address);
        inner.engine.reg_write(ArmRegister::LR, RUN_FUNCTION_LR);
        drop(inner);

        loop {
            let (pc, _) = self.read_pc_lr().unwrap();
            if pc == RUN_FUNCTION_LR {
                break;
            }

            self.run_some().await?;
        }

        let result = R::get(self);

        self.restore_context(&previous_context);

        Ok(result)
    }

    pub fn spawn<C, R, E>(&mut self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static,
        R: 'static,
        E: Debug + 'static,
    {
        let self_cloned = self.clone();
        self.inner.borrow_mut().system.spawn(move || SpawnFuture::new(self_cloned, callable));
    }

    pub fn register_function<F, P, E, R>(&mut self, function: F) -> ArmEngineResult<u32>
    where
        F: EmulatedFunction<P, E, R> + 'static,
        E: Debug + 'static,
        R: ResultWriter<R> + 'static,
        P: 'static,
    {
        let mut inner = self.inner.borrow_mut();

        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE as u64 + (inner.functions_count * 2) as u64;

        inner.engine.mem_write(address as u32, &bytes)?;

        let callback = RegisteredFunctionHolder::new(function);

        inner.functions.insert(address as u32, Rc::new(Box::new(callback)));
        inner.functions_count += 1;

        tracing::trace!("Register function at {:#x}", address);

        Ok(address as u32 + 1)
    }

    pub fn map(&mut self, address: u32, size: u32) -> ArmEngineResult<()> {
        tracing::trace!("Map address: {:#x}, size: {:#x}", address, size);

        let mut inner = self.inner.borrow_mut();

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
        let mut inner = self.inner.borrow_mut();

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
        let inner = self.inner.borrow();

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

    pub(crate) fn read_pc_lr(&self) -> ArmEngineResult<(u32, u32)> {
        let inner = self.inner.borrow();

        let lr = inner.engine.reg_read(ArmRegister::LR);
        let pc = inner.engine.reg_read(ArmRegister::PC);

        Ok((pc, lr))
    }

    pub(crate) fn write_result(&mut self, result: u32, lr: u32) -> ArmEngineResult<()> {
        let mut inner = self.inner.borrow_mut();

        inner.engine.reg_write(ArmRegister::R0, result);
        inner.engine.reg_write(ArmRegister::PC, lr);

        Ok(())
    }

    pub(crate) fn read_param(&self, pos: usize) -> ArmEngineResult<u32> {
        let inner = self.inner.borrow();

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
        let inner = self.inner.borrow();

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

        format!("{:#x}: {}\n", address, description)
    }

    fn dump_call_stack(&self, image_base: u32) -> ArmEngineResult<String> {
        let mut inner = self.inner.borrow_mut();

        let sp = inner.engine.reg_read(ArmRegister::SP);
        let pc = inner.engine.reg_read(ArmRegister::PC);
        let lr = inner.engine.reg_read(ArmRegister::LR);

        let mut call_stack = Self::format_callstack_address(pc, image_base);
        if lr != RUN_FUNCTION_LR && lr != 0 {
            call_stack += &Self::format_callstack_address(lr - 5, image_base);
        }

        for i in 0..128 {
            let address = sp + (i * 4);
            let value = inner.engine.mem_read(address, size_of::<u32>())?;
            let value_u32 = u32::from_le_bytes(value.try_into().unwrap());

            if value_u32 > 5 && Self::is_code_address(value_u32 - 4, image_base) {
                call_stack += &Self::format_callstack_address(value_u32 - 5, image_base);
            }
        }

        Ok(call_stack)
    }

    fn dump_stack(&self) -> ArmEngineResult<String> {
        let mut inner = self.inner.borrow_mut();

        let sp = inner.engine.reg_read(ArmRegister::SP);

        let mut result = String::new();
        for i in 0..16 {
            let address = sp + (i * 4);
            let value = inner.engine.mem_read(address, size_of::<u32>())?;
            let value_u32 = u32::from_le_bytes(value.try_into().unwrap());

            result += &format!("SP+{:#x}: {:#x}\n", i * 4, value_u32);
        }

        Ok(result)
    }
}

impl ByteRead for ArmCore {
    fn read_bytes(&self, address: u32, size: u32) -> anyhow::Result<Vec<u8>> {
        let mut inner = self.inner.borrow_mut();

        let data = inner.engine.mem_read(address, size as usize)?;

        // tracing::trace!("Read address: {:#x}, data: {:02x?}", address, data);

        Ok(data)
    }
}

impl ByteWrite for ArmCore {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()> {
        // tracing::trace!("Write address: {:#x}, data: {:02x?}", address, data);
        let mut inner = self.inner.borrow_mut();

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
