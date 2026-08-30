use wie_util::{ByteRead, ByteWrite, Result};

use crate::ArmCore;

const COPY_CHUNK: usize = 4096;
const STR_SCAN_CHUNK: usize = 256;

pub async fn memcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, len: u32) -> Result<()> {
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

pub async fn memset(core: &mut ArmCore, _: &mut (), ptr_dst: u32, value: u32, len: u32) -> Result<()> {
    let buf = [value as u8; COPY_CHUNK];
    let mut offset: u32 = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        offset = offset.wrapping_add(chunk as u32);
    }
    Ok(())
}

/// Reads in chunks because guest allocations are page-aligned. R0 already
/// holds the original `ptr_dst` for ARM ABI return, so the function returns
/// `()` and leaves R0 untouched.
pub async fn strcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    let mut buf = [0u8; STR_SCAN_CHUNK];
    let mut offset: u32 = 0;
    loop {
        core.read_bytes(ptr_src.wrapping_add(offset), &mut buf)?;
        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..=pos])?;
            return Ok(());
        }
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf)?;
        offset = offset.wrapping_add(STR_SCAN_CHUNK as u32);
    }
}

pub async fn strlen(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    let mut buf = [0u8; STR_SCAN_CHUNK];
    let mut len: u32 = 0;
    loop {
        core.read_bytes(ptr_str.wrapping_add(len), &mut buf)?;
        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            return Ok(len.wrapping_add(pos as u32));
        }
        len = len.wrapping_add(STR_SCAN_CHUNK as u32);
    }
}
