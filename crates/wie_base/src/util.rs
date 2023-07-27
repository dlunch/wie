use alloc::{string::String, vec::Vec};
use core::{mem::size_of, slice};

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
    T: Copy,
    R: ?Sized + ByteRead,
{
    let data = reader.read_bytes(address, size_of::<T>() as u32)?;

    Ok(unsafe { *(data.as_ptr() as *const T) })
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

    log::trace!("Read address: {:#x}, data: {:02x?}", address, result);

    Ok(String::from_utf8(result)?)
}

pub fn write_generic<W, T>(writer: &mut W, address: u32, data: T) -> anyhow::Result<()>
where
    W: ?Sized + ByteWrite,
{
    let data_slice = unsafe { slice::from_raw_parts(&data as *const T as *const u8, size_of::<T>()) };

    writer.write_bytes(address, data_slice)
}

pub fn cast_slice<T, U>(source: &[T]) -> &[U] {
    let new_len = core::mem::size_of_val(source) / core::mem::size_of::<U>();
    unsafe { slice::from_raw_parts(source as *const [T] as *const U, new_len) }
}

pub fn cast_vec<T, U>(source: Vec<T>) -> Vec<U> {
    let new_len = source.len() * core::mem::size_of::<T>() / core::mem::size_of::<U>();
    let new_capacity = source.capacity() * core::mem::size_of::<T>() / core::mem::size_of::<U>();

    let result = unsafe { Vec::from_raw_parts(source.as_ptr() as _, new_len, new_capacity) };
    core::mem::forget(source);

    result
}
