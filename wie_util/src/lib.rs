#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{
    any::Any,
    error::Error,
    fmt::{self, Display, Formatter},
    result,
};

use bytemuck::{AnyBitPattern, NoUninit, bytes_of, bytes_of_mut};

#[derive(Debug)]
pub enum WieError {
    InvalidMemoryAccess(u32),
    AllocationFailure,
    JavaException(u32), // to pass java exception down to rust
    Unimplemented(String),
    FatalError(String),
}

impl Display for WieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WieError::InvalidMemoryAccess(address) => write!(f, "Invalid memory access; address: {address}"),
            WieError::AllocationFailure => write!(f, "Allocation failure"),
            WieError::JavaException(exception) => write!(f, "Java exception: {exception:#x}"),
            WieError::Unimplemented(message) => write!(f, "Unimplemented: {message}"),
            WieError::FatalError(message) => write!(f, "Fatal error: {message}"),
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

pub fn read_null_terminated_string_bytes<R>(reader: &R, address: u32) -> Result<Vec<u8>>
where
    R: ?Sized + ByteRead,
{
    let mut result = Vec::with_capacity(20);
    let mut cursor = address;
    let mut buffer = [0; 4];
    loop {
        reader.read_bytes(cursor, &mut buffer)?;
        cursor += 4;

        // find zero in buffer
        let word = bytemuck::from_bytes::<u32>(&buffer);
        let has_zero_byte = word.overflowing_sub(0x01010101).0 & !(*word) & 0x80808080;

        if has_zero_byte != 0 {
            // zero byte location
            let zero_byte = has_zero_byte.trailing_zeros() as usize / 8;
            result.extend_from_slice(&buffer[..zero_byte]);
            break;
        } else {
            result.extend_from_slice(&buffer);
        }
    }

    // tracing::trace!("Read address: {:#x}, data: {:02x?}", address, result);

    Ok(result)
}

pub fn write_null_terminated_string_bytes<W>(writer: &mut W, address: u32, bytes: &[u8]) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    // TODO temp
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

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
