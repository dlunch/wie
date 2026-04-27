use wie_util::{ByteRead, ByteWrite, Result};

use crate::ArmCore;

const COPY_CHUNK: usize = 4096;

pub fn memcpy(core: &mut ArmCore, ptr_dst: u32, ptr_src: u32, len: u32) -> Result<()> {
    let mut buf = [0u8; COPY_CHUNK];
    let mut offset: u32 = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.read_bytes(ptr_src.wrapping_add(offset), &mut buf[..chunk])?;
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        offset = offset.wrapping_add(chunk as u32);
    }
    Ok(())
}

pub fn memset(core: &mut ArmCore, ptr_dst: u32, value: u8, len: u32) -> Result<()> {
    let buf = [value; COPY_CHUNK];
    let mut offset: u32 = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        offset = offset.wrapping_add(chunk as u32);
    }
    Ok(())
}

/// Returns the original `ptr_dst` so call sites that mirror the C ABI can write
/// it back to r0 without bookkeeping the input separately. Reads byte-by-byte
/// to avoid faulting past a mapped page when the NUL terminator is still
/// within range.
pub fn strcpy(core: &mut ArmCore, ptr_dst: u32, ptr_src: u32) -> Result<u32> {
    let mut offset: u32 = 0;
    let mut byte = [0u8; 1];
    loop {
        core.read_bytes(ptr_src.wrapping_add(offset), &mut byte)?;
        core.write_bytes(ptr_dst.wrapping_add(offset), &byte)?;
        if byte[0] == 0 {
            break;
        }
        offset = offset.wrapping_add(1);
    }
    Ok(ptr_dst)
}

pub fn strlen(core: &mut ArmCore, ptr_str: u32) -> Result<u32> {
    let mut len: u32 = 0;
    let mut byte = [0u8; 1];
    loop {
        core.read_bytes(ptr_str.wrapping_add(len), &mut byte)?;
        if byte[0] == 0 {
            break;
        }
        len = len.wrapping_add(1);
    }
    Ok(len)
}
