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
