use alloc::vec::Vec;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext, RUN_FUNCTION_LR};
use wie_util::{Result, read_generic, write_generic};

use crate::runtime::globals::SUPPORT_CONTEXT_BASE;

const FRAME_WORDS: u32 = 18;

// Each task owns this guest-memory state; the fixed ABI cell points at the active task.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct JavaSupportContext {
    ptr_current_exception_frame: u32,
    ptr_pending_exception: u32,
}

pub fn init(core: &mut ArmCore) -> Result<u32> {
    let ptr_context = Allocator::alloc(core, size_of::<JavaSupportContext>() as u32)?;
    write_generic(core, ptr_context, JavaSupportContext::zeroed())?;
    write_generic(core, SUPPORT_CONTEXT_BASE, ptr_context)?;
    Ok(ptr_context)
}

pub fn destroy(core: &mut ArmCore, ptr_context: u32) -> Result<()> {
    let context: JavaSupportContext = read_generic(core, ptr_context)?;
    let mut ptr_frame = context.ptr_current_exception_frame;
    while ptr_frame != 0 {
        let previous = read_generic(core, ptr_frame)?;
        Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)?;
        ptr_frame = previous;
    }
    Allocator::free(core, ptr_context, size_of::<JavaSupportContext>() as u32)
}

pub fn push(core: &mut ArmCore) -> Result<()> {
    let context = core.save_context();
    let ptr_context: u32 = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let mut support_context: JavaSupportContext = read_generic(core, ptr_context)?;
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
    write_generic(core, ptr_context, support_context)
}

pub fn pop(core: &mut ArmCore) -> Result<()> {
    let ptr_context: u32 = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let mut support_context: JavaSupportContext = read_generic(core, ptr_context)?;
    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, support_context.ptr_current_exception_frame)?;
    let ptr_frame = support_context.ptr_current_exception_frame;
    support_context.ptr_current_exception_frame = frame[0];
    support_context.ptr_pending_exception = 0;
    write_generic(core, ptr_context, support_context)?;
    Allocator::free(core, ptr_frame, FRAME_WORDS * size_of::<u32>() as u32)
}

pub fn pending(core: &ArmCore) -> Result<u32> {
    let ptr_context: u32 = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let support_context: JavaSupportContext = read_generic(core, ptr_context)?;
    Ok(support_context.ptr_pending_exception)
}

pub fn root_words(core: &ArmCore, ptr_context: u32, words: &mut Vec<u32>) -> Result<()> {
    let context: JavaSupportContext = read_generic(core, ptr_context)?;
    words.push(context.ptr_pending_exception);
    let mut ptr_frame = context.ptr_current_exception_frame;
    while ptr_frame != 0 {
        let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
        words.extend_from_slice(&frame[1..]);
        core.append_stack_words(frame[14], words)?;
        ptr_frame = frame[0];
    }
    Ok(())
}

pub fn unwind(core: &mut ArmCore, ptr_exception: u32) -> Result<Option<u32>> {
    // Rust callers must handle the exception before an enclosing guest catch can run.
    if core.read_pc_lr()?.1 == RUN_FUNCTION_LR {
        return Ok(None);
    }

    let ptr_context: u32 = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let mut support_context: JavaSupportContext = read_generic(core, ptr_context)?;
    if support_context.ptr_current_exception_frame == 0 {
        return Ok(None);
    }

    let ptr_frame = support_context.ptr_current_exception_frame;
    let frame: [u32; FRAME_WORDS as usize] = read_generic(core, ptr_frame)?;
    support_context.ptr_pending_exception = ptr_exception;
    write_generic(core, ptr_context, support_context)?;

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
