use alloc::{boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};
use core::{fmt::Debug, future::Future, marker::PhantomData};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{uc_error, Arch, HookType, MemType, Mode, Permission},
    RegisterARM, Unicorn,
};

use wie_base::{
    util::{round_up, ByteRead, ByteWrite},
    Core,
};

use crate::{
    context::ArmCoreContext,
    function::EmulatedFunction,
    future::{RunFunctionFuture, RunFunctionResult},
    Allocator,
};

const IMAGE_BASE: u32 = 0x100000;
const FUNCTIONS_BASE: u32 = 0x71000000;
pub const RUN_FUNCTION_LR: u32 = 0x7f000000;
pub const HEAP_BASE: u32 = 0x40000000;
pub const PEB_BASE: u32 = 0x7ff00000;

#[derive(Debug)]
pub struct UnicornError(uc_error);

impl From<UnicornError> for anyhow::Error {
    fn from(err: UnicornError) -> Self {
        anyhow::anyhow!("{:?}", err.0)
    }
}

pub type ArmCoreError = anyhow::Error;
pub type ArmCoreResult<T> = anyhow::Result<T>;

pub struct ArmCore {
    pub(crate) uc: Unicorn<'static, ()>,
    functions: BTreeMap<u32, Box<dyn RegisteredFunction>>,
    functions_count: usize,
}

impl ArmCore {
    pub fn new() -> ArmCoreResult<Self> {
        let mut uc = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).map_err(UnicornError)?;

        uc.add_code_hook(0, 0xffffffff, Self::code_hook).map_err(UnicornError)?;
        uc.add_mem_hook(HookType::MEM_INVALID, 0, 0xffff_ffff_ffff_ffff, Self::mem_hook)
            .map_err(UnicornError)?;

        uc.mem_map(FUNCTIONS_BASE as u64, 0x1000, Permission::READ).map_err(UnicornError)?;

        uc.reg_write(RegisterARM::CPSR, 0x40000010).map_err(UnicornError)?; // usr32

        Ok(Self {
            uc,
            functions: BTreeMap::new(),
            functions_count: 0,
        })
    }

    pub fn load(&mut self, data: &[u8], map_size: usize) -> ArmCoreResult<u32> {
        self.uc
            .mem_map(IMAGE_BASE as u64, round_up(map_size, 0x1000), Permission::ALL)
            .map_err(UnicornError)?;
        self.uc.mem_write(IMAGE_BASE as u64, data).map_err(UnicornError)?;

        Ok(IMAGE_BASE)
    }

    pub async fn run(&mut self, context: ArmCoreContext) -> ArmCoreResult<ArmCoreContext> {
        self.restore_context(&context)?;

        let pc = self.uc.reg_read(RegisterARM::PC).map_err(UnicornError)? as u32 + 1;
        let result = self.uc.emu_start(pc as u64, RUN_FUNCTION_LR as u64, 0, 100).map_err(UnicornError);

        if let Err(err) = &result {
            if err.0 == uc_error::FETCH_PROT {
                let cur_pc = self.uc.reg_read(RegisterARM::PC).map_err(UnicornError)? as u32;
                if (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000).contains(&cur_pc) {
                    let function = self.functions.remove(&cur_pc).unwrap();

                    function.call(self).await;

                    self.functions.insert(cur_pc, function);
                }
            } else {
                result?;
            }
        }

        self.save_context()
    }

    pub fn set_next(&mut self, address: u32, params: &[u32]) -> ArmCoreResult<()> {
        // is there cleaner way to do this?
        if !params.is_empty() {
            self.uc.reg_write(RegisterARM::R0, params[0] as u64).map_err(UnicornError)?;
        }
        if params.len() > 1 {
            self.uc.reg_write(RegisterARM::R1, params[1] as u64).map_err(UnicornError)?;
        }
        if params.len() > 2 {
            self.uc.reg_write(RegisterARM::R2, params[2] as u64).map_err(UnicornError)?;
        }
        if params.len() > 3 {
            self.uc.reg_write(RegisterARM::R3, params[3] as u64).map_err(UnicornError)?;
        }
        if params.len() > 4 {
            for param in params[4..].iter() {
                let sp = self.uc.reg_read(RegisterARM::SP).map_err(UnicornError)?;

                self.uc.reg_write(RegisterARM::SP, sp - 4).map_err(UnicornError)?;
                self.uc.mem_write(sp - 4, &param.to_le_bytes()).map_err(UnicornError)?;
            }
        }

        self.uc.reg_write(RegisterARM::PC, address as u64).map_err(UnicornError)?;

        Ok(())
    }

    pub fn run_function<R>(&mut self, address: u32, params: &[u32]) -> impl Future<Output = R>
    where
        R: RunFunctionResult<R>,
    {
        let previous_context = ArmCoreContext::from_uc(&self.uc);

        let stack_base = Allocator::alloc(self, 0x1000).unwrap();
        self.uc
            .reg_write(RegisterARM::SP, (stack_base + 0x1000) as u64)
            .map_err(UnicornError)
            .unwrap();

        self.set_next(address, params).unwrap();
        self.uc.reg_write(RegisterARM::LR, RUN_FUNCTION_LR as u64).unwrap();

        let context = ArmCoreContext::from_uc(&self.uc);

        let future = RunFunctionFuture::from_context(context, previous_context.clone());

        self.restore_context(&previous_context).unwrap();

        future
    }

    pub fn register_function<F, P, E, C, R>(&mut self, function: F, context: &C) -> ArmCoreResult<u32>
    where
        F: EmulatedFunction<P, E, C, R> + 'static,
        E: Debug + 'static,
        C: Clone + 'static,
        R: ResultWriter<R> + 'static,
        P: 'static,
    {
        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE as u64 + (self.functions_count * 2) as u64;

        self.uc.mem_write(address, &bytes).map_err(UnicornError)?;

        let callback = RegisteredFunctionHolder::new(function, context);

        self.functions.insert(address as u32, Box::new(callback));
        self.functions_count += 1;

        log::trace!("Register function at {:#x}", address);

        Ok(address as u32 + 1)
    }

    pub fn alloc(&mut self, address: u32, size: u32) -> ArmCoreResult<()> {
        log::trace!("Alloc address: {:#x}, size: {:#x}", address, size);

        self.uc
            .mem_map(address as u64, size as usize, Permission::READ | Permission::WRITE)
            .map_err(UnicornError)?;

        Ok(())
    }

    pub fn dump_regs(&self) -> ArmCoreResult<String> {
        Self::dump_regs_inner(&self.uc)
    }

    pub fn restore_context(&mut self, context: &ArmCoreContext) -> ArmCoreResult<()> {
        self.uc.reg_write(RegisterARM::R0, context.r0 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R1, context.r1 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R2, context.r2 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R3, context.r3 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R4, context.r4 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R5, context.r5 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R6, context.r6 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R7, context.r7 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::R8, context.r8 as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::SB, context.sb as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::SL, context.sl as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::FP, context.fp as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::IP, context.ip as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::SP, context.sp as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::LR, context.lr as u64).map_err(UnicornError)?;
        self.uc.reg_write(RegisterARM::PC, context.pc as u64).map_err(UnicornError)?;

        Ok(())
    }

    pub fn save_context(&mut self) -> ArmCoreResult<ArmCoreContext> {
        Ok(ArmCoreContext {
            r0: self.uc.reg_read(RegisterARM::R0).map_err(UnicornError)? as u32,
            r1: self.uc.reg_read(RegisterARM::R1).map_err(UnicornError)? as u32,
            r2: self.uc.reg_read(RegisterARM::R2).map_err(UnicornError)? as u32,
            r3: self.uc.reg_read(RegisterARM::R3).map_err(UnicornError)? as u32,
            r4: self.uc.reg_read(RegisterARM::R4).map_err(UnicornError)? as u32,
            r5: self.uc.reg_read(RegisterARM::R5).map_err(UnicornError)? as u32,
            r6: self.uc.reg_read(RegisterARM::R6).map_err(UnicornError)? as u32,
            r7: self.uc.reg_read(RegisterARM::R7).map_err(UnicornError)? as u32,
            r8: self.uc.reg_read(RegisterARM::R8).map_err(UnicornError)? as u32,
            sb: self.uc.reg_read(RegisterARM::SB).map_err(UnicornError)? as u32,
            sl: self.uc.reg_read(RegisterARM::SL).map_err(UnicornError)? as u32,
            fp: self.uc.reg_read(RegisterARM::FP).map_err(UnicornError)? as u32,
            ip: self.uc.reg_read(RegisterARM::IP).map_err(UnicornError)? as u32,
            sp: self.uc.reg_read(RegisterARM::SP).map_err(UnicornError)? as u32,
            lr: self.uc.reg_read(RegisterARM::LR).map_err(UnicornError)? as u32,
            pc: self.uc.reg_read(RegisterARM::PC).map_err(UnicornError)? as u32,
        })
    }

    fn dump_regs_inner(uc: &Unicorn<'_, ()>) -> ArmCoreResult<String> {
        let value = (|| {
            Ok::<_, uc_error>(
                [
                    format!(
                        "R0: {:#x} R1: {:#x} R2: {:#x} R3: {:#x} R4: {:#x} R5: {:#x} R6: {:#x} R7: {:#x} R8: {:#x}",
                        uc.reg_read(RegisterARM::R0)?,
                        uc.reg_read(RegisterARM::R1)?,
                        uc.reg_read(RegisterARM::R2)?,
                        uc.reg_read(RegisterARM::R3)?,
                        uc.reg_read(RegisterARM::R4)?,
                        uc.reg_read(RegisterARM::R5)?,
                        uc.reg_read(RegisterARM::R6)?,
                        uc.reg_read(RegisterARM::R7)?,
                        uc.reg_read(RegisterARM::R8)?,
                    ),
                    format!(
                        "SB: {:#x} SL: {:#x} FP: {:#x} IP: {:#x} SP: {:#x} LR: {:#x} PC: {:#x}",
                        uc.reg_read(RegisterARM::SB)?,
                        uc.reg_read(RegisterARM::SL)?,
                        uc.reg_read(RegisterARM::FP)?,
                        uc.reg_read(RegisterARM::IP)?,
                        uc.reg_read(RegisterARM::SP)?,
                        uc.reg_read(RegisterARM::LR)?,
                        uc.reg_read(RegisterARM::PC)?,
                    ),
                    format!("APSR: {:032b}\n", uc.reg_read(RegisterARM::APSR)?),
                ]
                .join("\n"),
            )
        })()
        .map_err(UnicornError)?;

        Ok(value)
    }

    fn code_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        let insn = uc.mem_read_as_vec(address, size as usize).unwrap();

        let cs = Capstone::new()
            .arm()
            .mode(capstone::arch::arm::ArchMode::Thumb)
            .detail(true)
            .build()
            .unwrap();

        let insns = cs.disasm_all(&insn, address).unwrap();

        let insn_str = insns
            .iter()
            .map(|x| format!("{:#x}: {} {}", x.address(), x.mnemonic().unwrap(), x.op_str().unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        log::trace!("{}\n{}", insn_str, Self::dump_regs_inner(uc).unwrap());
    }

    fn mem_hook(uc: &mut Unicorn<'_, ()>, mem_type: MemType, address: u64, size: usize, value: i64) -> bool {
        let pc = uc.reg_read(RegisterARM::PC).unwrap();
        let lr = uc.reg_read(RegisterARM::LR).unwrap();

        if mem_type == MemType::FETCH_PROT && pc == address && (FUNCTIONS_BASE..FUNCTIONS_BASE + 0x1000).contains(&(address as u32)) {
            return false;
        }

        if mem_type == MemType::READ || mem_type == MemType::FETCH || mem_type == MemType::WRITE {
            let value_str = if mem_type == MemType::WRITE {
                format!("{:#x}", value)
            } else {
                let value = uc.mem_read_as_vec(address, size).unwrap();

                if size == 4 {
                    format!("{:#x}", u32::from_le_bytes(value.try_into().unwrap()))
                } else {
                    format!("{:?}", value)
                }
            };

            log::trace!(
                "pc: {:#x} lr: {:#x} mem_type: {:?} address: {:#x} size: {:#x} value: {}",
                pc,
                lr,
                mem_type,
                address,
                size,
                value_str
            );

            true
        } else {
            log::error!(
                "Invalid Memory Access\n\
                mem_type: {:?} address: {:#x} size: {:#x} value: {:#x}\n{}",
                mem_type,
                address,
                size,
                value,
                Self::dump_regs_inner(uc).unwrap()
            );

            false
        }
    }
}

impl Core for ArmCore {}

impl ByteRead for ArmCore {
    fn read_bytes(&self, address: u32, size: u32) -> anyhow::Result<Vec<u8>> {
        let data = self.uc.mem_read_as_vec(address as u64, size as usize).map_err(UnicornError)?;

        log::trace!("Read address: {:#x}, data: {:02x?}", address, data);

        Ok(data)
    }
}

impl ByteWrite for ArmCore {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()> {
        log::trace!("Write address: {:#x}, data: {:02x?}", address, data);

        self.uc.mem_write(address as u64, data).map_err(UnicornError)?;

        Ok(())
    }
}

pub trait ResultWriter<R> {
    fn write(core: &mut ArmCore, value: R, lr: u32) -> anyhow::Result<()>;
}

impl ResultWriter<u32> for u32 {
    fn write(core: &mut ArmCore, value: u32, lr: u32) -> anyhow::Result<()> {
        core.uc.reg_write(RegisterARM::R0, value as u64).map_err(UnicornError)?;
        core.uc.reg_write(RegisterARM::PC, lr as u64).map_err(UnicornError)?;

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
trait RegisteredFunction {
    async fn call(&self, core: &mut ArmCore);
}

struct RegisteredFunctionHolder<F, P, E, C, R>
where
    F: EmulatedFunction<P, E, C, R> + 'static,
    E: Debug,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    function: Box<F>,
    context: C,
    _phantom: PhantomData<(P, E, C, R)>,
}

impl<F, P, E, C, R> RegisteredFunctionHolder<F, P, E, C, R>
where
    F: EmulatedFunction<P, E, C, R> + 'static,
    E: Debug,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    pub fn new(function: F, context: &C) -> Self {
        Self {
            function: Box::new(function),
            context: context.clone(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<F, P, E, C, R> RegisteredFunction for RegisteredFunctionHolder<F, P, E, C, R>
where
    F: EmulatedFunction<P, E, C, R> + 'static,
    E: Debug,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    async fn call(&self, core: &mut ArmCore) {
        let lr = core.uc.reg_read(RegisterARM::LR).unwrap() as u32;
        let pc = core.uc.reg_read(RegisterARM::PC).unwrap() as u32;
        log::debug!("Registered function called at {:#x}, LR: {:#x}", pc, lr);

        let mut new_context = self.context.clone();

        let result = self.function.call(core, &mut new_context).await.unwrap();
        R::write(core, result, lr).unwrap();
    }
}
