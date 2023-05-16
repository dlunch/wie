use wie_backend::Task;
use wie_core_arm::{Allocator, ArmCore, ArmCoreContext};

const STACK_SIZE: u32 = 0x1000;
const TASK_LR: u32 = 0x7f000000;

pub struct KtfTask {
    core: ArmCore,
    context: ArmCoreContext,
}

impl KtfTask {
    pub fn from_pc_args(core: &mut ArmCore, pc: u32, args: &[u32]) -> anyhow::Result<Self> {
        let stack_base = Allocator::alloc(core, STACK_SIZE)? + STACK_SIZE;
        let mut context = ArmCoreContext {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            sb: 0,
            sl: 0,
            fp: 0,
            ip: 0,
            sp: stack_base,
            lr: TASK_LR,
            pc,
        };

        if !args.is_empty() {
            context.r0 = args[0];
        }
        if args.len() > 1 {
            context.r1 = args[1];
        }
        if args.len() > 2 {
            context.r2 = args[2];
        }
        if args.len() > 3 {
            context.r3 = args[3];
        }
        if args.len() > 4 {
            todo!()
        }

        Ok(Self { core: core.clone(), context })
    }
}

impl Task for KtfTask {
    fn run_some(&mut self) -> anyhow::Result<()> {
        self.core.restore_context(&self.context)?;

        self.core.run_some(TASK_LR, 100)?;

        self.context = self.core.save_context()?;

        Ok(())
    }

    fn is_finished(&self) -> bool {
        self.context.pc == TASK_LR
    }
}
