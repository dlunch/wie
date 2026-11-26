use bytemuck::{Pod, Zeroable};

use wie_util::{Result, WieError, read_generic, write_generic};
use wie_wipi_c::WIPICContext;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Pool {
    next: u32,
    end: u32,
}

pub async fn reset(context: &mut dyn WIPICContext, base: u32, size: u32) -> Result<()> {
    tracing::debug!("MXUserMemInterf[0]({base:#x}, {size})");

    let header_size = size_of::<Pool>() as u32;
    if size < header_size {
        return Err(WieError::AllocationFailure);
    }
    let end = base.checked_add(size).ok_or(WieError::AllocationFailure)?;
    write_generic(
        context,
        base,
        Pool {
            next: base + header_size,
            end,
        },
    )
}

pub async fn alloc(context: &mut dyn WIPICContext, base: u32, size: u32) -> Result<u32> {
    tracing::debug!("MXUserMemInterf[1]({base:#x}, {size})");

    let mut pool: Pool = read_generic(context, base)?;
    let Some(start) = pool.next.checked_next_multiple_of(align_of::<u32>() as u32) else {
        return Ok(0);
    };
    if start > pool.end || size > pool.end - start {
        return Ok(0);
    }
    pool.next = start + size;
    write_generic(context, base, pool)?;
    Ok(start)
}
