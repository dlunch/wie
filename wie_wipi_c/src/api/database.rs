use alloc::{borrow::ToOwned, boxed::Box, str, string::String, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::Database;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::{WIPICWord, context::WIPICContext};

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct DatabaseHandle {
    name: [u8; 32], // TODO hardcoded max size
}

pub async fn open_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, record_size: i32, create: i32, mode: i32) -> Result<i32> {
    tracing::debug!("MC_dbOpenDataBase({:#x}, {}, {}, {})", ptr_name, record_size, create, mode);

    let name = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?).unwrap();

    if record_size == 1 {
        // TODO: is parameter record_size correct??
        return Ok(-12); // M_E_NOENT
    }

    let name_bytes = name.as_bytes();
    let mut handle = DatabaseHandle { name: [0; 32] };

    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    let ptr_handle = context.alloc_raw(size_of::<DatabaseHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;

    tracing::debug!("Created database handle {ptr_handle:#x} for {name}");

    Ok(ptr_handle as _)
}

pub async fn close_database(context: &mut dyn WIPICContext, db_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbCloseDataBase({:#x})", db_id);

    if db_id < 0x10000 {
        // TODO some apps store database id in 16bit, so we need to handle it
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    context.free_raw(db_id as _, size_of::<DatabaseHandle>() as _)?;

    Ok(0) // success
}

pub async fn list_record(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecords({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    let db = get_database_from_db_id(context, db_id);
    let ids = db.get_record_ids();

    let mut cursor = 0;
    for &id in &ids {
        write_generic(context, buf_ptr + cursor, id)?;
        cursor += size_of::<WIPICWord>() as u32;
    }

    Ok(ids.len() as _)
}

pub async fn write_record_single(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_db_write_record_single({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    let mut buf = vec![0; buf_len as _];
    context.read_bytes(buf_ptr, &mut buf)?;
    let mut db = get_database_from_db_id(context, db_id);

    db.set(1, &buf);

    Ok(1)
}

pub async fn delete_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({:#x}, {})", db_id, rec_id);

    let mut db = get_database_from_db_id(context, db_id);

    let result = db.delete(rec_id as _);

    if result {
        Ok(0) // success
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn read_record_single(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_db_read_record_single({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    if db_id < 0x10000 {
        // TODO some apps store database id in 16bit, so we need to handle it
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let db = get_database_from_db_id(context, db_id);

    if let Some(x) = db.get(1) {
        if buf_len < x.len() as _ {
            return Ok(-18); // M_E_SHORTBUF
        }
        context.write_bytes(buf_ptr, &x)?;

        Ok(0)
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn select_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({:#x}, {}, {:#x}, {})", db_id, rec_id, buf_ptr, buf_len);

    let db = get_database_from_db_id(context, db_id);

    if let Some(x) = db.get(rec_id as _) {
        if buf_len < x.len() as _ {
            return Ok(-18); // M_E_SHORTBUF
        }
        context.write_bytes(buf_ptr, &x)?;

        Ok(0)
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn unk16(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_dbUnk16()");

    Ok(1)
}

fn get_database_from_db_id(context: &mut dyn WIPICContext, db_id: i32) -> Box<dyn Database> {
    let handle: DatabaseHandle = read_generic(context, db_id as _).unwrap();

    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let db_name = str::from_utf8(&handle.name[..name_length]).unwrap();
    let app_id = context.system().app_id().to_owned();

    context.system().platform().database_repository().open(db_name, &app_id)
}
