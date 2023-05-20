use core::cell::RefCell;

use wie_base::{
    task::{Task, TaskStatus},
    Core,
};

use crate::{core::ArmCoreContext, Allocator, ArmCore};

const STACK_SIZE: u32 = 0x1000;
const TASK_LR: u32 = 0x7f000000;

pub struct ArmCoreTask {
    context: RefCell<ArmCoreContext>,
}

impl ArmCoreTask {
    pub fn from_pc_args(core: &mut ArmCore, pc: u32, args: &[u32]) -> anyhow::Result<Self> {
        log::debug!("Creating task from pc {:08x} with args {:?}", pc, args);

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

        Ok(Self {
            context: RefCell::new(context),
        })
    }
}

impl Task for ArmCoreTask {
    fn run(&self, core: &mut dyn Core) -> anyhow::Result<()> {
        let core = core.as_any_mut().downcast_mut::<ArmCore>().unwrap();
        core.restore_context(&self.context.borrow())?;

        core.run_some(TASK_LR)?;

        self.context.replace(core.save_context()?);

        Ok(())
    }

    fn status(&self) -> TaskStatus {
        if self.context.borrow().pc == TASK_LR {
            TaskStatus::Finished
        } else {
            TaskStatus::Running
        }
    }

    fn sleep(&self, _: &mut dyn Core, _: u64) {
        todo!()
    }

    fn r#yield(&self, _: &mut dyn Core) {
        todo!()
    }
}
