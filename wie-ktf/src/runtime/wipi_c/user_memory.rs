use bytemuck::{Pod, Zeroable};

use wie_util::{Result, read_generic, write_generic};
use wie_wipi_c::WIPICContext;

// The interface only receives the caller's buffer, so the arena state lives at its start.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Pool {
    next: u32,
    end: u32,
}

pub async fn reset(context: &mut dyn WIPICContext, base: u32, size: u32) -> Result<()> {
    tracing::debug!("MXUserMemInterf[0]({base:#x}, {size})");

    write_generic(
        context,
        base,
        Pool {
            next: base + size_of::<Pool>() as u32,
            end: base + size,
        },
    )
}

pub async fn alloc(context: &mut dyn WIPICContext, base: u32, size: u32) -> Result<u32> {
    tracing::debug!("MXUserMemInterf[1]({base:#x}, {size})");

    let mut pool: Pool = read_generic(context, base)?;
    let start = pool.next.next_multiple_of(align_of::<u32>() as u32);
    if size > pool.end.saturating_sub(start) {
        return Ok(0);
    }
    pool.next = start + size;
    write_generic(context, base, pool)?;

    Ok(start)
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::{TestPlatform, run_jvm_test_with_system};
    use wie_backend::Options;
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;
    use wie_wipi_c::WIPICContext;

    use super::{alloc, reset};
    use crate::runtime::wipi_c::context::KtfWIPICContext;

    #[test]
    fn arena_allocates_aligned_blocks_until_reset() -> Result<()> {
        run_jvm_test_with_system(Box::new([]), Box::new(TestPlatform::new()), |jvm, system| async move {
            let mut core = ArmCore::new(Options::default()).unwrap();
            Allocator::init(&mut core).unwrap();
            let mut context = KtfWIPICContext::new(core, system, jvm);

            let base = context.alloc_raw(32).unwrap();
            reset(&mut context, base, 32).await.unwrap();
            let first = alloc(&mut context, base, 3).await.unwrap();
            assert!((base..base + 32).contains(&first));
            let second = alloc(&mut context, base, 1).await.unwrap();
            assert_eq!(second, first + 4);
            assert_eq!(alloc(&mut context, base, 32).await.unwrap(), 0);
            reset(&mut context, base, 32).await.unwrap();
            assert_eq!(alloc(&mut context, base, 3).await.unwrap(), first);

            Ok(())
        })
    }
}
