#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{
    error::Error,
    fmt::{self, Display, Formatter},
    result,
};

use bytemuck::{bytes_of, bytes_of_mut, AnyBitPattern, NoUninit};

#[derive(Debug)]
pub enum WieError {
    InvalidMemoryAccess(u32),
    AllocationFailure,
    Unimplemented(String),
    FatalError(String),
}

impl Display for WieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WieError::InvalidMemoryAccess(address) => write!(f, "Invalid memory access; address: {}", address),
            WieError::AllocationFailure => write!(f, "Allocation failure"),
            WieError::Unimplemented(message) => write!(f, "Unimplemented: {}", message),
            WieError::FatalError(message) => write!(f, "Fatal error: {}", message),
        }
    }
}

impl Error for WieError {}

pub type Result<T> = result::Result<T, WieError>;

pub trait ByteRead {
    fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize>;
}

pub trait ByteWrite {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> Result<()>;
}

pub fn read_generic<T, R>(reader: &R, address: u32) -> Result<T>
where
    T: Copy + AnyBitPattern + NoUninit,
    R: ?Sized + ByteRead,
{
    unsafe {
        #[allow(clippy::uninit_assumed_init)] // XXX
        let destination = &mut core::mem::MaybeUninit::<T>::uninit().assume_init();

        reader.read_bytes(address, bytes_of_mut(destination))?;

        Ok(*destination)
    }
}

pub fn read_null_terminated_string<R>(reader: &R, address: u32) -> Result<String>
where
    R: ?Sized + ByteRead,
{
    // TODO we can read by 4bytes at once

    let mut result = Vec::new();
    let mut cursor = address;
    loop {
        let mut item = [0u8; 1];
        reader.read_bytes(cursor, &mut item)?;
        cursor += 1;

        if item[0] == 0 {
            break;
        }

        result.push(item[0]);
    }

    // tracing::trace!("Read address: {:#x}, data: {:02x?}", address, result);

    // TODO temp
    Ok(encoding_rs::EUC_KR.decode(&result).0.into())

    // Ok(String::from_utf8(result).unwrap())
}

pub fn write_null_terminated_string<W>(writer: &mut W, address: u32, string: &str) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    let bytes = string.as_bytes();

    writer.write_bytes(address, bytes)?;
    writer.write_bytes(address + bytes.len() as u32, &[0])?;

    // tracing::trace!("Write address: {:#x}, data: {:02x?}", address, bytes);

    Ok(())
}

pub fn write_generic<W, T>(writer: &mut W, address: u32, data: T) -> Result<()>
where
    W: ?Sized + ByteWrite,
    T: NoUninit,
{
    let data_slice = bytes_of(&data);

    writer.write_bytes(address, data_slice)
}

pub fn read_null_terminated_table<R>(reader: &R, base_address: u32) -> Result<Vec<u32>>
where
    R: ?Sized + ByteRead,
{
    let mut cursor = base_address;
    let mut result = Vec::new();
    loop {
        let item: u32 = read_generic(reader, cursor)?;
        if item == 0 {
            break;
        }
        result.push(item);

        cursor += 4;
    }

    Ok(result)
}

pub fn write_null_terminated_table<W>(writer: &mut W, base_address: u32, items: &[u32]) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    let mut cursor = base_address;
    for &item in items {
        write_generic(writer, cursor, item)?;

        cursor += 4;
    }
    write_generic(writer, cursor, 0u32)
}
