use alloc::{borrow::ToOwned, boxed::Box, str, string::String, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::WIPICWord;

use wie_backend::Database;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::context::WIPICContext;

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct DatabaseHandle {
    name: [u8; 32], // TODO hardcoded max size
}

pub async fn open_database(context: &mut dyn WIPICContext, ptr_name: WIPICWord, mode: i32, r#type: i32) -> Result<i32> {
    tracing::debug!("MC_dbOpenDataBase({ptr_name:#x}, {mode}, {}", r#type);

    let name = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?).unwrap();

    let system = context.system();
    let pid = system.pid().to_owned();

    let exists = system.platform().database_repository().exists(system, &name, &pid).await;

    if !exists && mode == 1 {
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
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    context.free_raw(db_id as _, size_of::<DatabaseHandle>() as _)?;

    Ok(0) // success
}

pub async fn list_record(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbListRecords({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    let db = get_database_from_db_id(context, db_id).await;
    let ids = db.get_record_ids().await;

    let mut cursor = 0;
    for &id in &ids {
        write_generic(context, buf_ptr + cursor, id)?;
        cursor += size_of::<WIPICWord>() as u32;
    }

    Ok(ids.len() as _)
}

pub async fn write_record_single(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_db_write_record_single({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let mut buf = vec![0; buf_len as _];
    context.read_bytes(buf_ptr, &mut buf)?;
    let mut db = get_database_from_db_id(context, db_id).await;

    db.set(1, &buf).await;

    Ok(buf_len as _)
}

pub async fn delete_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32) -> Result<i32> {
    tracing::debug!("MC_dbDeleteRecord({:#x}, {})", db_id, rec_id);

    let mut db = get_database_from_db_id(context, db_id).await;

    let result = db.delete(rec_id as _).await;

    if result {
        Ok(0) // success
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn read_record_single(context: &mut dyn WIPICContext, db_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_db_read_record_single({:#x}, {:#x}, {})", db_id, buf_ptr, buf_len);

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let db = get_database_from_db_id(context, db_id).await;

    if let Some(x) = db.get(1).await {
        if buf_len < x.len() as _ {
            return Ok(-18); // M_E_SHORTBUF
        }
        context.write_bytes(buf_ptr, &x)?;

        Ok(x.len() as _)
    } else {
        Ok(-22) // M_E_BADRECID
    }
}

pub async fn select_record(context: &mut dyn WIPICContext, db_id: i32, rec_id: i32, buf_ptr: WIPICWord, buf_len: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_dbSelectRecord({:#x}, {}, {:#x}, {})", db_id, rec_id, buf_ptr, buf_len);

    if db_id < 0x10000 {
        return Ok(-25); // M_E_INVALIDHANDLE
    }

    let db = get_database_from_db_id(context, db_id).await;

    if let Some(x) = db.get(rec_id as _).await {
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

async fn get_database_from_db_id(context: &mut dyn WIPICContext, db_id: i32) -> Box<dyn Database> {
    let handle: DatabaseHandle = read_generic(context, db_id as _).unwrap();

    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let db_name = str::from_utf8(&handle.name[..name_length]).unwrap();

    let system = context.system();
    let pid = system.pid().to_owned();

    system.platform().database_repository().open(system, db_name, &pid).await
}
