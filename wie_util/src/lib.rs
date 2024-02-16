#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::mem::size_of;

use bytemuck::{bytes_of, from_bytes, AnyBitPattern, NoUninit};

pub fn round_up(num_to_round: usize, multiple: usize) -> usize {
    if multiple == 0 {
        return num_to_round;
    }

    let remainder = num_to_round % multiple;
    if remainder == 0 {
        num_to_round
    } else {
        num_to_round + multiple - remainder
    }
}

pub trait ByteRead {
    fn read_bytes(&self, address: u32, size: u32) -> anyhow::Result<Vec<u8>>;
}

pub trait ByteWrite {
    fn write_bytes(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()>;
}

pub fn read_generic<T, R>(reader: &R, address: u32) -> anyhow::Result<T>
where
    T: Copy + AnyBitPattern,
    R: ?Sized + ByteRead,
{
    let data = reader.read_bytes(address, size_of::<T>() as u32)?;

    Ok(*from_bytes(&data))
}

pub fn read_null_terminated_string<R>(reader: &R, address: u32) -> anyhow::Result<String>
where
    R: ?Sized + ByteRead,
{
    // TODO we can read by 4bytes at once

    let mut result = Vec::new();
    let mut cursor = address;
    loop {
        let item = reader.read_bytes(cursor, 1)?;
        cursor += 1;

        if item[0] == 0 {
            break;
        }

        result.push(item[0]);
    }

    // tracing::trace!("Read address: {:#x}, data: {:02x?}", address, result);

    Ok(String::from_utf8(result).unwrap())
}

pub fn write_null_terminated_string<W>(writer: &mut W, address: u32, string: &str) -> anyhow::Result<()>
where
    W: ?Sized + ByteWrite,
{
    let bytes = string.as_bytes();

    writer.write_bytes(address, bytes)?;
    writer.write_bytes(address + bytes.len() as u32, &[0])?;

    // tracing::trace!("Write address: {:#x}, data: {:02x?}", address, bytes);

    Ok(())
}

pub fn write_generic<W, T>(writer: &mut W, address: u32, data: T) -> anyhow::Result<()>
where
    W: ?Sized + ByteWrite,
    T: NoUninit,
{
    let data_slice = bytes_of(&data);

    writer.write_bytes(address, data_slice)
}

pub fn read_null_terminated_table<R>(reader: &R, base_address: u32) -> anyhow::Result<Vec<u32>>
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

pub fn write_null_terminated_table<W>(writer: &mut W, base_address: u32, items: &[u32]) -> anyhow::Result<()>
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
