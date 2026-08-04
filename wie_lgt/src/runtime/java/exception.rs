use core::mem::size_of;

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext};
use wie_util::{Result, read_generic, write_generic};

const STATE_WORDS: u32 = 2;
const FRAME_WORDS: u32 = 18;

#[derive(Clone, Copy)]
pub struct JavaExceptionState {
    ptr_raw: u32,
}

impl JavaExceptionState {
    pub fn new(core: &mut ArmCore) -> Result<Self> {
        let ptr_raw = Allocator::alloc(core, STATE_WORDS * size_of::<u32>() as u32)?;
        write_generic(core, ptr_raw, [0u32; STATE_WORDS as usize])?;

        Ok(Self { ptr_raw })
    }

    pub fn push(&self, core: &mut ArmCore) -> Result<()> {
        let context = core.save_context();
        let ptr_previous_frame: u32 = read_generic(core, self.ptr_raw)?;
        let frame = [
            ptr_previous_frame,
            context.r0,
            context.r1,
            context.r2,
            context.r3,
            context.r4,
            context.r5,
            context.r6,
            context.r7,
            context.r8,
            context.sb,
            context.sl,
            context.fp,
            context.ip,
            context.sp,
            context.lr,
            context.pc,
            context.cpsr,
        ];
        let ptr_frame = Allocator::alloc(core, FRAME_WORDS * size_of::<u32>() as u32)?;
        write_generic(core, ptr_frame, frame)?;
        write_generic(core, self.ptr_raw, ptr_frame)?;
        write_generic(core, self.ptr_raw + size_of::<u32>() as u32, 0u32)?;

        Ok(())
    }

    pub fn pop(&self, core: &mut ArmCore) -> Result<()> {
        let ptr_frame: u32 = read_generic(core, self.ptr_raw)?;
        let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
        write_generic(core, self.ptr_raw, frame[0])?;
        write_generic(core, self.ptr_raw + size_of::<u32>() as u32, 0u32)?;
        Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)
    }

    pub fn pending(&self, core: &ArmCore) -> Result<u32> {
        read_generic(core, self.ptr_raw + size_of::<u32>() as u32)
    }

    pub fn unwind(&self, core: &mut ArmCore, ptr_exception: u32) -> Result<Option<u32>> {
        let ptr_frame: u32 = read_generic(core, self.ptr_raw)?;
        if ptr_frame == 0 {
            return Ok(None);
        }

        let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
        write_generic(core, self.ptr_raw + size_of::<u32>() as u32, ptr_exception)?;

        let context = ArmCoreContext {
            r0: frame[1],
            r1: frame[2],
            r2: frame[3],
            r3: frame[4],
            r4: frame[5],
            r5: frame[6],
            r6: frame[7],
            r7: frame[8],
            r8: frame[9],
            sb: frame[10],
            sl: frame[11],
            fp: frame[12],
            ip: frame[13],
            sp: frame[14],
            lr: frame[15],
            pc: frame[16],
            cpsr: frame[17],
        };
        core.restore_context(&context);
        core.set_next_pc(context.lr)?;

        Ok(Some(context.lr))
    }
}

#[cfg(test)]
mod tests {
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use super::JavaExceptionState;

    #[test]
    fn exception_frame_restores_guest_context() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        let state = JavaExceptionState::new(&mut core)?;

        let mut context = core.save_context();
        context.r4 = 0x44;
        context.sp = 0x12000;
        context.lr = 0x4001;
        core.restore_context(&context);
        state.push(&mut core)?;

        context.r4 = 0;
        context.lr = 0;
        core.restore_context(&context);
        assert_eq!(state.unwind(&mut core, 0x1234)?, Some(0x4001));

        let restored = core.save_context();
        assert_eq!(restored.r4, 0x44);
        assert_eq!(restored.sp, 0x12000);
        assert_eq!(restored.pc, 0x4000);
        assert_eq!(state.pending(&core)?, 0x1234);

        state.pop(&mut core)?;
        assert_eq!(state.pending(&core)?, 0);
        assert_eq!(state.unwind(&mut core, 0x5678)?, None);

        Ok(())
    }
}
