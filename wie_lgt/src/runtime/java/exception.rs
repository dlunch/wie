use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext};
use wie_util::{Result, read_generic, write_generic};

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;
const FRAME_WORDS: u32 = 18;

// Fixed guest-memory context shared by the LGT Java SVC handlers.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaSupportContext {
    ptr_current_exception_frame: u32,
    ptr_pending_exception: u32,
}

pub fn init(core: &mut ArmCore) -> Result<()> {
    write_generic(core, SUPPORT_CONTEXT_BASE, JavaSupportContext::zeroed())
}

pub fn push(core: &mut ArmCore) -> Result<()> {
    let context = core.save_context();
    let mut support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let frame = [
        support_context.ptr_current_exception_frame,
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
    support_context.ptr_current_exception_frame = ptr_frame;
    support_context.ptr_pending_exception = 0;
    write_generic(core, SUPPORT_CONTEXT_BASE, support_context)
}

pub fn pop(core: &mut ArmCore) -> Result<()> {
    let mut support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, support_context.ptr_current_exception_frame)?;
    let ptr_frame = support_context.ptr_current_exception_frame;
    support_context.ptr_current_exception_frame = frame[0];
    support_context.ptr_pending_exception = 0;
    write_generic(core, SUPPORT_CONTEXT_BASE, support_context)?;
    Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)
}

pub fn pending(core: &ArmCore) -> Result<u32> {
    let support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    Ok(support_context.ptr_pending_exception)
}

pub fn unwind(core: &mut ArmCore, ptr_exception: u32) -> Result<Option<u32>> {
    let mut support_context: JavaSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    if support_context.ptr_current_exception_frame == 0 {
        return Ok(None);
    }

    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, support_context.ptr_current_exception_frame)?;
    support_context.ptr_pending_exception = ptr_exception;
    write_generic(core, SUPPORT_CONTEXT_BASE, support_context)?;

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

#[cfg(test)]
mod tests {
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use super::{init, pending, pop, push, unwind};

    #[test]
    fn exception_frame_restores_guest_context() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        init(&mut core)?;

        let mut context = core.save_context();
        context.r4 = 0x44;
        context.sp = 0x12000;
        context.lr = 0x4001;
        core.restore_context(&context);
        push(&mut core)?;

        context.r4 = 0;
        context.lr = 0;
        core.restore_context(&context);
        assert_eq!(unwind(&mut core, 0x1234)?, Some(0x4001));

        let restored = core.save_context();
        assert_eq!(restored.r4, 0x44);
        assert_eq!(restored.sp, 0x12000);
        assert_eq!(restored.pc, 0x4000);
        assert_eq!(pending(&core)?, 0x1234);

        pop(&mut core)?;
        assert_eq!(pending(&core)?, 0);
        assert_eq!(unwind(&mut core, 0x5678)?, None);

        Ok(())
    }
}
