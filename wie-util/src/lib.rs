#![no_std]
extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use core::{
    any::Any,
    error::Error,
    fmt::{self, Display, Formatter},
    mem::{MaybeUninit, size_of},
    result,
    slice::from_raw_parts_mut,
};

use bytemuck::{AnyBitPattern, NoUninit, bytes_of};

#[derive(Debug)]
pub enum WieError {
    InvalidMemoryAccess(u32),
    AllocationFailure,
    JavaException(u32), // to pass java exception down to rust
    JavaExceptionUnwind { context_base: u32, target: u32, next_pc: u32 },
    Unimplemented(String),
    FatalError(String),
}

impl Display for WieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WieError::InvalidMemoryAccess(address) => write!(f, "Invalid memory access; address: {address}"),
            WieError::AllocationFailure => write!(f, "Allocation failure"),
            WieError::JavaException(exception) => write!(f, "Java exception: {exception:#x}"),
            WieError::JavaExceptionUnwind {
                context_base,
                target,
                next_pc,
            } => write!(
                f,
                "Java exception unwind: context_base={context_base:#x}, target={target:#x}, next_pc={next_pc:#x}"
            ),
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
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let mut destination = MaybeUninit::<T>::uninit();
    let destination_bytes = unsafe { from_raw_parts_mut(destination.as_mut_ptr().cast::<u8>(), size_of::<T>()) };
    let read = reader.read_bytes(address, destination_bytes)?;
    if read != destination_bytes.len() {
        return Err(WieError::FatalError(format!(
            "Short read at {address:#x}: expected {}, got {read}",
            destination_bytes.len()
        )));
    }

    Ok(unsafe { destination.assume_init() })
}

pub fn read_null_terminated_string_bytes<R>(reader: &R, address: u32) -> Result<Vec<u8>>
where
    R: ?Sized + ByteRead,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let mut result = Vec::with_capacity(20);
    let mut cursor = address;
    let mut byte = [0; 1];
    loop {
        let read = reader.read_bytes(cursor, &mut byte)?;
        if read != 1 {
            return Err(WieError::FatalError(format!("Short read at {cursor:#x}: expected 1, got {read}")));
        }

        if byte[0] == 0 {
            break;
        }

        result.push(byte[0]);
        cursor += 1;
    }

    Ok(result)
}

pub fn write_null_terminated_string_bytes<W>(writer: &mut W, address: u32, bytes: &[u8]) -> Result<()>
where
    W: ?Sized + ByteWrite,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    // TODO temp
    writer.write_bytes(address, bytes)?;
    writer.write_bytes(address + bytes.len() as u32, &[0])?;

    Ok(())
}

pub fn write_generic<W, T>(writer: &mut W, address: u32, data: T) -> Result<()>
where
    W: ?Sized + ByteWrite,
    T: NoUninit,
{
    if address == 0 {
        return Err(WieError::InvalidMemoryAccess(address));
    }

    let data_slice = bytes_of(&data);

    writer.write_bytes(address, data_slice)
}

pub fn read_null_terminated_table<R>(reader: &R, base_address: u32) -> Result<Vec<u32>>
where
    R: ?Sized + ByteRead,
{
    if base_address == 0 {
        return Err(WieError::InvalidMemoryAccess(base_address));
    }

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
    if base_address == 0 {
        return Err(WieError::InvalidMemoryAccess(base_address));
    }

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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    struct StrictMemory {
        memory: Vec<u8>,
    }

    impl ByteRead for StrictMemory {
        fn read_bytes(&self, address: u32, result: &mut [u8]) -> Result<usize> {
            let address = address as usize;
            let end = address + result.len();
            if end > self.memory.len() {
                return Err(WieError::InvalidMemoryAccess(address as u32));
            }

            result.copy_from_slice(&self.memory[address..end]);

            Ok(result.len())
        }
    }

    #[test]
    fn read_generic_reads_into_initialized_storage() {
        let memory = StrictMemory {
            memory: vec![0, 0x78, 0x56, 0x34, 0x12],
        };

        let value: u32 = read_generic(&memory, 1).unwrap();

        assert_eq!(value, 0x1234_5678);
    }

    #[test]
    fn read_null_terminated_string_handles_four_byte_boundaries() {
        let memory = StrictMemory {
            memory: vec![0, b't', b'e', b's', b't', 0],
        };

        let value = read_null_terminated_string_bytes(&memory, 1).unwrap();

        assert_eq!(value, b"test");
    }
}
